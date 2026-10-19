use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::{ManagedPack, PackSource};
use crate::archive::{self, safe_relative_path};
use crate::instance::{Instance, NewInstance, create_instance};
use crate::minecraft::{LoaderKind, LoaderSpec};
use crate::net::{Download, DownloadProgress, HttpClient};
use crate::{Error, Result};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MrpackIndex {
    pub name: String,
    pub version_id: String,
    pub files: Vec<MrpackFile>,
    pub dependencies: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MrpackFile {
    pub path: String,
    pub hashes: HashMap<String, String>,
    #[serde(default)]
    pub env: Option<HashMap<String, String>>,
    pub downloads: Vec<String>,
    pub file_size: u64,
}

impl MrpackFile {
    fn is_client_file(&self) -> bool {
        self.env
            .as_ref()
            .and_then(|env| env.get("client"))
            .is_none_or(|support| support != "unsupported")
    }
}

impl MrpackIndex {
    pub fn minecraft_version(&self) -> Result<&str> {
        self.dependencies
            .get("minecraft")
            .map(String::as_str)
            .ok_or_else(|| {
                Error::InvalidInput("modpack does not specify a Minecraft version".into())
            })
    }

    pub fn loader(&self) -> Result<Option<LoaderSpec>> {
        for (key, kind) in [
            ("fabric-loader", LoaderKind::Fabric),
            ("quilt-loader", LoaderKind::Quilt),
        ] {
            if let Some(version) = self.dependencies.get(key) {
                return Ok(Some(LoaderSpec {
                    kind,
                    version: version.clone(),
                }));
            }
        }
        if let Some(loader) = ["forge", "neoforge"]
            .into_iter()
            .find(|key| self.dependencies.contains_key(*key))
        {
            return Err(Error::InvalidInput(format!(
                "{loader} modpacks are not supported yet"
            )));
        }
        Ok(None)
    }

    fn downloads(&self, game_dir: &Path) -> Result<Vec<Download>> {
        self.files
            .iter()
            .filter(|file| file.is_client_file())
            .map(|file| {
                let relative = safe_relative_path(&file.path).ok_or_else(|| {
                    Error::InvalidInput(format!("unsafe path in modpack: {}", file.path))
                })?;
                let url = file
                    .downloads
                    .first()
                    .ok_or_else(|| Error::InvalidInput(format!("no download for {}", file.path)))?;
                Ok(Download {
                    url: url.clone(),
                    path: game_dir.join(relative),
                    sha1: file.hashes.get("sha1").cloned(),
                    size: Some(file.file_size),
                })
            })
            .collect()
    }
}

pub fn read_mrpack_index(archive_path: &Path) -> Result<MrpackIndex> {
    let file = std::fs::File::open(archive_path).map_err(|err| Error::io(archive_path, err))?;
    let mut zip = zip::ZipArchive::new(file).map_err(|err| Error::archive(archive_path, err))?;
    let mut entry = zip
        .by_name("modrinth.index.json")
        .map_err(|err| Error::archive(archive_path, err))?;
    let mut text = String::new();
    entry
        .read_to_string(&mut text)
        .map_err(|err| Error::io(archive_path, err))?;
    serde_json::from_str(&text).map_err(|err| Error::json(archive_path, err))
}

/// Creates a new instance from a `.mrpack` archive: downloads its files and applies overrides.
/// The half-created instance is removed again if anything fails.
pub async fn install_mrpack(
    http: &HttpClient,
    archive_path: &Path,
    instances_dir: &Path,
    name: Option<String>,
    source: Option<PackSource>,
    concurrency: usize,
    on_progress: impl Fn(DownloadProgress),
) -> Result<Instance> {
    let index = read_mrpack_index(archive_path)?;
    let mut instance = create_instance(
        instances_dir,
        NewInstance {
            name: name.unwrap_or_else(|| index.name.clone()),
            minecraft_version: index.minecraft_version()?.to_owned(),
            loader: index.loader()?,
            group: None,
            icon_key: None,
        },
    )?;
    let result = populate(
        http,
        archive_path,
        &index,
        &instance,
        concurrency,
        on_progress,
    )
    .await;
    if let Err(err) = result {
        let _ = std::fs::remove_dir_all(&instance.root);
        return Err(err);
    }
    instance.config.managed_pack = source.map(|source| ManagedPack {
        provider: source.provider,
        project_id: source.project_id,
        version_id: source.version_id,
        name: index.name.clone(),
        version: index.version_id.clone(),
    });
    instance.save()?;
    Ok(instance)
}

async fn populate(
    http: &HttpClient,
    archive_path: &Path,
    index: &MrpackIndex,
    instance: &Instance,
    concurrency: usize,
    on_progress: impl Fn(DownloadProgress),
) -> Result<()> {
    let game_dir = instance.game_dir();
    http.download_all(index.downloads(&game_dir)?, concurrency, on_progress)
        .await?;
    // Client overrides are applied last so they win over the shared ones.
    for prefix in ["overrides", "client-overrides"] {
        archive::extract(archive_path, &game_dir, |name| {
            name.strip_prefix(prefix).ok().map(PathBuf::from)
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn index(json: &str) -> MrpackIndex {
        serde_json::from_str(json).unwrap_or_else(|err| panic!("{err}"))
    }

    #[test]
    fn maps_dependencies_to_game_and_loader() -> Result<()> {
        let pack = index(
            r#"{"formatVersion":1,"game":"minecraft","versionId":"8.2.0","name":"FO","files":[],
                "dependencies":{"minecraft":"1.21.4","fabric-loader":"0.19.3"}}"#,
        );
        assert_eq!(pack.minecraft_version()?, "1.21.4");
        assert_eq!(
            pack.loader()?,
            Some(LoaderSpec {
                kind: LoaderKind::Fabric,
                version: "0.19.3".into()
            })
        );
        let forge = index(
            r#"{"versionId":"1","name":"F","files":[],"dependencies":{"minecraft":"1.20.1","forge":"47.2.0"}}"#,
        );
        assert!(forge.loader().is_err());
        Ok(())
    }

    #[test]
    fn skips_server_only_files_and_rejects_unsafe_paths() -> Result<()> {
        let pack = index(
            r#"{"versionId":"1","name":"P","dependencies":{"minecraft":"1.21.4"},"files":[
                {"path":"mods/a.jar","hashes":{"sha1":"aa"},"downloads":["https://x/a"],"fileSize":1},
                {"path":"mods/server.jar","hashes":{},"env":{"client":"unsupported","server":"required"},"downloads":["https://x/s"],"fileSize":1}]}"#,
        );
        let downloads = pack.downloads(Path::new("/game"))?;
        assert_eq!(downloads.len(), 1);
        assert_eq!(downloads[0].path, Path::new("/game/mods/a.jar"));

        let evil = index(
            r#"{"versionId":"1","name":"P","dependencies":{},"files":[
                {"path":"../../.bashrc","hashes":{},"downloads":["https://x"],"fileSize":1}]}"#,
        );
        assert!(evil.downloads(Path::new("/game")).is_err());
        Ok(())
    }
}
