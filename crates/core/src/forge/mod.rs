//! Installs Forge and NeoForge (1.13+) the way their installers do: fetch the libraries,
//! then run the installer's processors to produce the patched game jar.

mod profile;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tokio::process::Command;

use self::profile::{InstallProfile, Processor, Variables};
use crate::minecraft::{LoaderKind, LoaderProfile, LoaderSpec};
use crate::net::{Download, DownloadProgress, HttpClient, sha1_of};
use crate::paths::DataPaths;
use crate::{Error, Result, archive, fs};

const FORGE_MAVEN: &str = "https://maven.minecraftforge.net";
const NEOFORGE_MAVEN: &str = "https://maven.neoforged.net/releases";
/// The data entry that names the patched game jar, the installer's final product.
const PATCHED_KEY: &str = "PATCHED";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallProgress {
    Downloading(DownloadProgress),
    /// Running the installer's processors, which report no progress of their own.
    Processing,
}

pub struct ForgeInstaller<'a> {
    pub http: &'a HttpClient,
    pub paths: &'a DataPaths,
    pub java: &'a Path,
    pub concurrency: usize,
}

/// What a finished installation left behind, so it is not repeated on every launch.
#[derive(Serialize, Deserialize)]
struct InstalledLoader {
    version_json: serde_json::Value,
    required_files: Vec<PathBuf>,
}

impl ForgeInstaller<'_> {
    /// Installs the loader if needed and returns its version profile.
    pub async fn install(
        &self,
        loader: &LoaderSpec,
        game_version: &str,
        client_jar: &Path,
        on_progress: impl Fn(InstallProgress),
    ) -> Result<LoaderProfile> {
        let marker = self
            .paths
            .meta()
            .join("loaders")
            .join(loader.kind.id())
            .join(format!("{game_version}-{}.json", loader.version));
        if let Some(installed) = fs::read_json::<InstalledLoader>(&marker)?
            && installed.required_files.iter().all(|file| file.is_file())
        {
            return parse_profile(installed.version_json, &marker);
        }

        log::info!(
            "installing {} {} for {game_version}",
            loader.kind.id(),
            loader.version
        );
        let installer = self.download_installer(loader, game_version).await?;
        let profile = read_install_profile(&installer)?;
        let version_json: serde_json::Value = serde_json::from_str(&archive::read_entry_to_string(
            &installer,
            profile.json.trim_start_matches('/'),
        )?)
        .map_err(|err| Error::json(&installer, err))?;
        let loader_profile = parse_profile(version_json.clone(), &installer)?;

        self.fetch_libraries(&installer, &profile, &loader_profile, |progress| {
            on_progress(InstallProgress::Downloading(progress));
        })
        .await?;
        on_progress(InstallProgress::Processing);
        let work_dir =
            self.paths
                .downloads_cache()
                .join(format!("{}-{}", loader.kind.id(), loader.version));
        let result = self
            .run_processors(&installer, &profile, game_version, client_jar, &work_dir)
            .await;
        let _ = std::fs::remove_dir_all(&work_dir);
        let required_files = result?;

        fs::write_json(
            &marker,
            &InstalledLoader {
                version_json,
                required_files,
            },
        )?;
        Ok(loader_profile)
    }

    async fn download_installer(&self, loader: &LoaderSpec, game_version: &str) -> Result<PathBuf> {
        let (maven, coordinates) = match loader.kind {
            LoaderKind::Forge => (
                FORGE_MAVEN,
                format!(
                    "net.minecraftforge:forge:{game_version}-{}:installer",
                    loader.version
                ),
            ),
            LoaderKind::NeoForge => (
                NEOFORGE_MAVEN,
                format!("net.neoforged:neoforge:{}:installer", loader.version),
            ),
            LoaderKind::Fabric | LoaderKind::Quilt => {
                return Err(Error::InvalidInput(format!(
                    "{} has no installer",
                    loader.kind.id()
                )));
            }
        };
        let relative = crate::minecraft::maven_path(&coordinates, None)
            .ok_or_else(|| Error::InvalidInput(format!("invalid installer {coordinates}")))?;
        let path = self.paths.libraries().join(&relative);
        self.http
            .download(&Download {
                url: format!("{maven}/{relative}"),
                path: path.clone(),
                sha1: None,
                size: None,
            })
            .await?;
        Ok(path)
    }

    /// Downloads every library the installer and the game need; jars without a URL ship
    /// inside the installer or are produced by the processors.
    async fn fetch_libraries(
        &self,
        installer: &Path,
        profile: &InstallProfile,
        loader_profile: &LoaderProfile,
        on_progress: impl Fn(DownloadProgress),
    ) -> Result<()> {
        let libraries_dir = self.paths.libraries();
        let bundled: Vec<String> = archive::entry_names(installer)?;
        let mut downloads = Vec::new();
        for artifact in profile
            .libraries
            .iter()
            .chain(&loader_profile.libraries)
            .filter_map(crate::minecraft::Library::artifact)
        {
            let Some(relative) = artifact.path.clone() else {
                continue;
            };
            let path = libraries_dir.join(&relative);
            if !artifact.url.is_empty() {
                downloads.push(Download {
                    url: artifact.url.clone(),
                    path,
                    sha1: Some(artifact.sha1.clone()).filter(|sha1| !sha1.is_empty()),
                    size: Some(artifact.size).filter(|&size| size > 0),
                });
            } else if bundled
                .iter()
                .any(|name| *name == format!("maven/{relative}"))
            {
                archive::extract(installer, &libraries_dir, |name| {
                    name.strip_prefix("maven")
                        .ok()
                        .filter(|inner| *inner == Path::new(&relative))
                        .map(Path::to_path_buf)
                })?;
            }
        }
        self.http
            .download_all(downloads, self.concurrency, on_progress)
            .await
    }

    /// Runs the client processors and returns the files a complete installation must have.
    async fn run_processors(
        &self,
        installer: &Path,
        profile: &InstallProfile,
        game_version: &str,
        client_jar: &Path,
        work_dir: &Path,
    ) -> Result<Vec<PathBuf>> {
        let libraries_dir = self.paths.libraries();
        let path_string = |path: &Path| path.to_string_lossy().into_owned();
        let mut vars = Variables::new(
            &libraries_dir,
            HashMap::from([
                ("SIDE".to_owned(), "client".to_owned()),
                ("MINECRAFT_JAR".to_owned(), path_string(client_jar)),
                ("MINECRAFT_VERSION".to_owned(), game_version.to_owned()),
                ("ROOT".to_owned(), path_string(work_dir)),
                ("INSTALLER".to_owned(), path_string(installer)),
                ("LIBRARY_DIR".to_owned(), path_string(&libraries_dir)),
            ]),
        );
        for (key, value) in &profile.data {
            let resolved = self.resolve_data(installer, &vars, &value.client, work_dir)?;
            vars.insert(key, resolved);
        }
        for processor in profile.processors.iter().filter(|p| p.runs_on_client()) {
            self.run_processor(processor, &vars).await?;
        }
        Ok(vars
            .get(PATCHED_KEY)
            .map(PathBuf::from)
            .into_iter()
            .collect())
    }

    /// Data values are library coordinates, quoted literals or files inside the installer.
    fn resolve_data(
        &self,
        installer: &Path,
        vars: &Variables,
        value: &str,
        work_dir: &Path,
    ) -> Result<String> {
        if let Some(literal) = value.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')) {
            return Ok(literal.to_owned());
        }
        if value.starts_with('[') {
            return vars.resolve(value);
        }
        let entry = value.trim_start_matches('/');
        archive::extract(installer, work_dir, |name| {
            (name == Path::new(entry)).then(|| name.to_path_buf())
        })?;
        Ok(work_dir.join(entry).to_string_lossy().into_owned())
    }

    async fn run_processor(&self, processor: &Processor, vars: &Variables) -> Result<()> {
        let outputs = processor
            .outputs
            .iter()
            .map(|(file, sha1)| Ok((PathBuf::from(vars.resolve(file)?), vars.resolve(sha1)?)))
            .collect::<Result<Vec<_>>>()?;
        if !outputs.is_empty() && outputs_match(&outputs).await? {
            return Ok(());
        }

        let jar = vars.library(&processor.jar)?;
        let main_class = main_class(&jar)?;
        let mut classpath = vec![jar];
        for library in &processor.classpath {
            classpath.push(vars.library(library)?);
        }
        let classpath = std::env::join_paths(&classpath)
            .map_err(|err| Error::InvalidInput(format!("invalid processor classpath: {err}")))?;
        let args = processor
            .args
            .iter()
            .map(|arg| vars.resolve(arg))
            .collect::<Result<Vec<_>>>()?;

        let output = Command::new(self.java)
            .arg("-cp")
            .arg(classpath)
            .arg(&main_class)
            .args(&args)
            .output()
            .await
            .map_err(|err| Error::io(self.java, err))?;
        if !output.status.success() {
            let log = String::from_utf8_lossy(&output.stderr);
            let tail: Vec<&str> = log.lines().rev().take(5).collect();
            return Err(Error::InvalidInput(format!(
                "installer step {main_class} failed: {}",
                tail.into_iter().rev().collect::<Vec<_>>().join("\n")
            )));
        }
        if !outputs_match(&outputs).await? {
            return Err(Error::InvalidInput(format!(
                "installer step {main_class} produced unexpected files"
            )));
        }
        Ok(())
    }
}

fn read_install_profile(installer: &Path) -> Result<InstallProfile> {
    let text = archive::read_entry_to_string(installer, "install_profile.json")?;
    let raw: serde_json::Value =
        serde_json::from_str(&text).map_err(|err| Error::json(installer, err))?;
    if raw.get("install").is_some() {
        return Err(Error::InvalidInput(
            "Forge for Minecraft 1.12.2 and older is not supported yet".into(),
        ));
    }
    serde_json::from_value(raw).map_err(|err| Error::json(installer, err))
}

fn parse_profile(version_json: serde_json::Value, source: &Path) -> Result<LoaderProfile> {
    serde_json::from_value(version_json).map_err(|err| Error::json(source, err))
}

fn main_class(jar: &Path) -> Result<String> {
    archive::read_entry_to_string(jar, "META-INF/MANIFEST.MF")?
        .lines()
        .find_map(|line| line.strip_prefix("Main-Class:"))
        .map(|class| class.trim().to_owned())
        .ok_or_else(|| Error::InvalidInput(format!("{} has no main class", jar.display())))
}

async fn outputs_match(outputs: &[(PathBuf, String)]) -> Result<bool> {
    for (file, expected) in outputs {
        if !file.is_file() || !sha1_of(file).await?.eq_ignore_ascii_case(expected) {
            return Ok(false);
        }
    }
    Ok(true)
}
