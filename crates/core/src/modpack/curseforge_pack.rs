use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{ManagedPack, ModpackTarget, PackInstall, PackSource};
use crate::archive;
use crate::instance::{Instance, NewInstance, create_instance};
use crate::minecraft::{LoaderKind, LoaderSpec};
use crate::modplatform::curseforge::CurseForge;
use crate::net::{Download, DownloadProgress, HttpClient};
use crate::{Error, Result};

pub const CURSEFORGE_MANIFEST: &str = "manifest.json";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseManifest {
    pub minecraft: ManifestMinecraft,
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub files: Vec<ManifestFile>,
    #[serde(default = "default_overrides")]
    pub overrides: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestMinecraft {
    pub version: String,
    #[serde(default)]
    pub mod_loaders: Vec<ManifestLoader>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ManifestLoader {
    pub id: String,
    #[serde(default)]
    pub primary: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ManifestFile {
    #[serde(rename = "projectID")]
    pub project_id: u64,
    #[serde(rename = "fileID")]
    pub file_id: u64,
    #[serde(default = "required_by_default")]
    pub required: bool,
}

fn default_overrides() -> String {
    "overrides".into()
}

fn required_by_default() -> bool {
    true
}

/// A pack file whose author only allows downloads from the CurseForge website.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct BlockedFile {
    pub project_name: String,
    pub file_name: String,
    /// Page to download the file from by hand.
    pub url: String,
    /// Folder inside the game directory the file belongs in, e.g. `mods`.
    pub install_dir: String,
}

impl CurseManifest {
    /// The primary loader, e.g. `forge-47.2.0` becomes Forge 47.2.0.
    pub fn loader(&self) -> Result<Option<LoaderSpec>> {
        let Some(loader) = self
            .minecraft
            .mod_loaders
            .iter()
            .find(|loader| loader.primary)
            .or(self.minecraft.mod_loaders.first())
        else {
            return Ok(None);
        };
        let (name, version) = loader
            .id
            .split_once('-')
            .ok_or_else(|| Error::InvalidInput(format!("unknown mod loader {}", loader.id)))?;
        let kind = match name {
            "forge" => LoaderKind::Forge,
            "neoforge" => LoaderKind::NeoForge,
            "fabric" => LoaderKind::Fabric,
            "quilt" => LoaderKind::Quilt,
            _ => {
                return Err(Error::InvalidInput(format!(
                    "unsupported mod loader {name}"
                )));
            }
        };
        Ok(Some(LoaderSpec {
            kind,
            version: version.to_owned(),
        }))
    }
}

pub fn read_curseforge_manifest(archive_path: &Path) -> Result<CurseManifest> {
    let text = archive::read_entry_to_string(archive_path, CURSEFORGE_MANIFEST)?;
    serde_json::from_str(&text).map_err(|err| Error::json(archive_path, err))
}

/// Creates a new instance from a CurseForge modpack archive. Files that cannot be downloaded
/// automatically are returned so the user can fetch them by hand.
pub async fn install_curseforge_pack(
    curseforge: &CurseForge,
    http: &HttpClient,
    archive_path: &Path,
    target: &ModpackTarget<'_>,
    source: Option<PackSource>,
    on_progress: impl Fn(DownloadProgress),
) -> Result<PackInstall> {
    let manifest = read_curseforge_manifest(archive_path)?;
    let mut instance = create_instance(
        target.instances_dir,
        NewInstance {
            name: target.name.clone().unwrap_or_else(|| manifest.name.clone()),
            minecraft_version: manifest.minecraft.version.clone(),
            loader: manifest.loader()?,
            group: None,
            icon_key: None,
        },
    )?;
    let populated = populate(
        curseforge,
        http,
        archive_path,
        &manifest,
        &instance,
        target.concurrency,
        on_progress,
    )
    .await;
    let blocked = match populated {
        Ok(blocked) => blocked,
        Err(err) => {
            let _ = std::fs::remove_dir_all(&instance.root);
            return Err(err);
        }
    };
    instance.config.managed_pack = source.map(|source| ManagedPack {
        provider: source.provider,
        project_id: source.project_id,
        version_id: source.version_id,
        name: manifest.name.clone(),
        version: manifest.version.clone(),
    });
    instance.save()?;
    Ok(PackInstall { instance, blocked })
}

async fn populate(
    curseforge: &CurseForge,
    http: &HttpClient,
    archive_path: &Path,
    manifest: &CurseManifest,
    instance: &Instance,
    concurrency: usize,
    on_progress: impl Fn(DownloadProgress),
) -> Result<Vec<BlockedFile>> {
    let required: Vec<&ManifestFile> = manifest.files.iter().filter(|file| file.required).collect();
    let file_ids: Vec<u64> = required.iter().map(|file| file.file_id).collect();
    let mut project_ids: Vec<u64> = required.iter().map(|file| file.project_id).collect();
    project_ids.dedup();
    let projects: HashMap<String, _> = curseforge
        .projects(&project_ids)
        .await?
        .into_iter()
        .map(|project| (project.id.clone(), project))
        .collect();

    let game_dir = instance.game_dir();
    let mut downloads = Vec::new();
    let mut blocked = Vec::new();
    for version in curseforge.files(&file_ids).await? {
        let (Some(file), Some(project)) =
            (version.primary_file(), projects.get(&version.project_id))
        else {
            continue;
        };
        if file.url.is_empty() {
            blocked.push(BlockedFile {
                project_name: project.name.clone(),
                file_name: file.file_name.clone(),
                url: format!("{}/files/{}", project.website_url, version.id),
                install_dir: project.install_dir.to_owned(),
            });
        } else {
            downloads.push(Download {
                path: game_dir.join(project.install_dir).join(&file.file_name),
                ..file.download_into(&game_dir)
            });
        }
    }
    http.download_all(downloads, concurrency, on_progress)
        .await?;
    let prefix = manifest.overrides.trim_matches('/').to_owned();
    archive::extract(archive_path, &game_dir, |name| {
        name.strip_prefix(&prefix).ok().map(PathBuf::from)
    })?;
    blocked.sort_by(|a, b| a.project_name.cmp(&b.project_name));
    Ok(blocked)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_primary_loader() -> std::result::Result<(), serde_json::Error> {
        let manifest: CurseManifest = serde_json::from_str(
            r#"{"minecraft":{"version":"1.20.1","modLoaders":[{"id":"fabric-0.15.0"},{"id":"forge-47.2.0","primary":true}]},
                "manifestType":"minecraftModpack","name":"Pack","version":"1.0",
                "files":[{"projectID":1,"fileID":2,"required":true}],"overrides":"overrides"}"#,
        )?;
        assert_eq!(
            manifest.loader().ok().flatten(),
            Some(LoaderSpec {
                kind: LoaderKind::Forge,
                version: "47.2.0".into()
            })
        );
        assert_eq!(manifest.files[0].file_id, 2);
        Ok(())
    }
}
