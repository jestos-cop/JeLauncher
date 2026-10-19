use std::path::{Path, PathBuf};

use md5::{Digest, Md5};
use serde::Deserialize;

use super::{ManagedPack, ModpackTarget, PackInstall};
use crate::archive;
use crate::instance::{DraftGuard, Instance, NewInstance, create_instance};
use crate::minecraft::{LoaderKind, LoaderSpec};
use crate::modplatform::Provider;
use crate::modplatform::technic::{Technic, TechnicFiles};
use crate::net::{Download, DownloadProgress, HttpClient};
use crate::{Error, Result};

/// Technic packs ship their loader as a version file, either loose or inside this jar.
const MODPACK_JAR: &str = "bin/modpack.jar";
const VERSION_FILE: &str = "bin/version.json";

/// Creates a new instance from a Technic pack version: downloads its zip or Solder mods,
/// extracts them into the game directory and picks the loader from the pack's version file.
pub async fn install_technic_pack(
    technic: &Technic,
    http: &HttpClient,
    slug: &str,
    version: &str,
    target: &ModpackTarget<'_>,
    on_progress: impl Fn(DownloadProgress),
) -> Result<PackInstall> {
    let pack = technic.pack_version(slug, version).await?;
    let mut instance = create_instance(
        target.instances_dir,
        NewInstance {
            name: target
                .name
                .clone()
                .unwrap_or_else(|| pack.pack_name.clone()),
            minecraft_version: pack.minecraft_version.clone(),
            loader: None,
            group: None,
            icon_key: None,
        },
    )?;
    let draft = DraftGuard::new(&instance);
    let staging = target.download_dir.join(format!("technic-{}", instance.id));
    let populated = populate(http, &pack.files, &staging, &instance, target, on_progress).await;
    let _ = std::fs::remove_dir_all(&staging);
    populated?;
    apply_game_setup(&mut instance)?;
    instance.config.managed_pack = Some(ManagedPack {
        provider: Provider::Technic,
        project_id: slug.to_owned(),
        version_id: version.to_owned(),
        name: pack.pack_name,
        version: pack.version,
    });
    instance.save()?;
    draft.keep();
    Ok(PackInstall::complete(instance))
}

async fn populate(
    http: &HttpClient,
    files: &TechnicFiles,
    staging: &Path,
    instance: &Instance,
    target: &ModpackTarget<'_>,
    on_progress: impl Fn(DownloadProgress),
) -> Result<()> {
    let archives: Vec<(Download, Option<&str>)> = match files {
        TechnicFiles::Zip(url) => vec![(
            Download {
                url: url.clone(),
                path: staging.join("pack.zip"),
                sha1: None,
                size: None,
            },
            None,
        )],
        TechnicFiles::Solder(mods) => mods
            .iter()
            .enumerate()
            .map(|(index, module)| {
                let download = Download {
                    url: module.url.clone(),
                    path: staging.join(format!("{index}-{}.zip", module.name)),
                    sha1: None,
                    size: module.size,
                };
                (download, Some(module.md5.as_str()))
            })
            .collect(),
    };
    http.download_all(
        archives
            .iter()
            .map(|(download, _)| download.clone())
            .collect(),
        target.concurrency,
        on_progress,
    )
    .await?;
    let game_dir = instance.game_dir();
    for (download, md5) in &archives {
        if let Some(expected) = md5 {
            verify_md5(&download.path, expected)?;
        }
        archive::extract(&download.path, &game_dir, |name| Some(PathBuf::from(name)))?;
    }
    Ok(())
}

fn verify_md5(path: &Path, expected: &str) -> Result<()> {
    let bytes = std::fs::read(path).map_err(|err| Error::io(path, err))?;
    let actual = hex::encode(Md5::digest(&bytes));
    if actual.eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        Err(Error::Checksum {
            path: path.to_path_buf(),
            expected: expected.to_owned(),
            actual,
        })
    }
}

/// Reads the Minecraft version and loader from the extracted pack.
fn apply_game_setup(instance: &mut Instance) -> Result<()> {
    let game_dir = instance.game_dir();
    let modpack_jar = game_dir.join(MODPACK_JAR);
    let version_file = game_dir.join(VERSION_FILE);
    let (text, fml_minecraft, source) = if modpack_jar.is_file() {
        let entries = archive::entry_names(&modpack_jar)?;
        if !entries.iter().any(|name| name == "version.json") {
            return Err(Error::InvalidInput(
                "this pack patches the game jar, which is not supported".into(),
            ));
        }
        let fml_minecraft = entries
            .iter()
            .any(|name| name == "fmlversion.properties")
            .then(|| archive::read_entry_to_string(&modpack_jar, "fmlversion.properties"))
            .transpose()?
            .and_then(|properties| property(&properties, "fmlbuild.mcversion"));
        (
            archive::read_entry_to_string(&modpack_jar, "version.json")?,
            fml_minecraft,
            modpack_jar,
        )
    } else if version_file.is_file() {
        let text =
            std::fs::read_to_string(&version_file).map_err(|err| Error::io(&version_file, err))?;
        (text, None, version_file)
    } else {
        return Ok(());
    };
    let version: PackVersionFile =
        serde_json::from_str(&text).map_err(|err| Error::json(&source, err))?;
    if let Some(minecraft) = version.inherits_from.clone().or(fml_minecraft) {
        instance.config.minecraft_version = minecraft;
    }
    instance.config.loader = version.loader();
    Ok(())
}

fn property(properties: &str, key: &str) -> Option<String> {
    properties.lines().find_map(|line| {
        let (name, value) = line.split_once('=')?;
        (name.trim() == key).then(|| value.trim().to_owned())
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PackVersionFile {
    inherits_from: Option<String>,
    #[serde(default)]
    libraries: Vec<PackLibrary>,
    #[serde(default)]
    arguments: Option<PackArguments>,
}

#[derive(Deserialize)]
struct PackLibrary {
    name: String,
}

#[derive(Deserialize)]
struct PackArguments {
    #[serde(default)]
    game: Vec<serde_json::Value>,
}

impl PackVersionFile {
    fn loader(&self) -> Option<LoaderSpec> {
        let spec = |kind, version: &str| LoaderSpec {
            kind,
            version: version.to_owned(),
        };
        self.libraries.iter().find_map(|library| {
            let name = library.name.as_str();
            let coordinate_version = name.splitn(3, ':').nth(2).unwrap_or_default();
            if name.starts_with("net.neoforged.fancymodloader:") {
                self.neoforge_version()
                    .map(|version| spec(LoaderKind::NeoForge, &version))
            } else if (name.starts_with("net.minecraftforge:forge:")
                || name.starts_with("net.minecraftforge:fmlloader:"))
                && coordinate_version.contains('-')
            {
                // `1.20.1-47.2.0`, or `1.7.10-10.13.4.1614-1.7.10` on 1.7.10.
                coordinate_version
                    .split('-')
                    .nth(1)
                    .map(|version| spec(LoaderKind::Forge, version))
            } else if name.starts_with("net.minecraftforge:minecraftforge:") {
                Some(spec(LoaderKind::Forge, coordinate_version))
            } else if name.starts_with("net.fabricmc:fabric-loader:") {
                Some(spec(LoaderKind::Fabric, coordinate_version))
            } else if name.starts_with("org.quiltmc:quilt-loader:") {
                Some(spec(LoaderKind::Quilt, coordinate_version))
            } else {
                None
            }
        })
    }

    fn neoforge_version(&self) -> Option<String> {
        let game = &self.arguments.as_ref()?.game;
        game.windows(2).find_map(|pair| {
            let flag = pair[0].as_str()?;
            (flag == "--fml.neoForgeVersion" || flag == "--fml.forgeVersion")
                .then(|| pair[1].as_str().map(str::to_owned))
                .flatten()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn loader(json: &str) -> Option<LoaderSpec> {
        serde_json::from_str::<PackVersionFile>(json)
            .unwrap_or_else(|err| panic!("{err}"))
            .loader()
    }

    #[test]
    fn detects_loaders_from_version_files() {
        assert_eq!(
            loader(r#"{"libraries":[{"name":"net.minecraftforge:forge:1.12.2-14.23.5.2860"}]}"#),
            Some(LoaderSpec {
                kind: LoaderKind::Forge,
                version: "14.23.5.2860".into()
            })
        );
        assert_eq!(
            loader(
                r#"{"libraries":[{"name":"net.minecraftforge:forge:1.7.10-10.13.4.1614-1.7.10"}]}"#
            )
            .map(|spec| spec.version),
            Some("10.13.4.1614".into())
        );
        assert_eq!(
            loader(
                r#"{"libraries":[{"name":"org.ow2:asm:9"},{"name":"net.fabricmc:fabric-loader:0.16.9"}]}"#
            ),
            Some(LoaderSpec {
                kind: LoaderKind::Fabric,
                version: "0.16.9".into()
            })
        );
        assert_eq!(
            loader(
                r#"{"libraries":[{"name":"net.neoforged.fancymodloader:loader:4.0.24"}],
                    "arguments":{"game":["--fml.neoForgeVersion","21.1.77","--launchTarget","x"]}}"#
            ),
            Some(LoaderSpec {
                kind: LoaderKind::NeoForge,
                version: "21.1.77".into()
            })
        );
        assert_eq!(loader(r#"{"libraries":[]}"#), None);
    }

    #[test]
    fn reads_properties() {
        assert_eq!(
            property(
                "fmlbuild.major=7\nfmlbuild.mcversion = 1.7.10\n",
                "fmlbuild.mcversion"
            ),
            Some("1.7.10".into())
        );
    }
}
