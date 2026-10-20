use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{ManagedPack, ModpackTarget, PackInstall, PackSource};
use crate::archive;
use crate::instance::{DraftGuard, Instance, NewInstance, create_instance};
use crate::minecraft::{LoaderKind, LoaderSpec};
use crate::modplatform::ModPlatforms;
use crate::net::{Download, DownloadProgress};
use crate::{Error, Result};

pub const CURSEFORGE_MANIFEST: &str = "manifest.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseManifest {
    pub minecraft: ManifestMinecraft,
    #[serde(default = "manifest_type")]
    pub manifest_type: String,
    #[serde(default = "manifest_version")]
    pub manifest_version: u32,
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub files: Vec<ManifestFile>,
    #[serde(default = "default_overrides")]
    pub overrides: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestMinecraft {
    pub version: String,
    #[serde(default)]
    pub mod_loaders: Vec<ManifestLoader>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestLoader {
    pub id: String,
    #[serde(default)]
    pub primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestFile {
    #[serde(rename = "projectID")]
    pub project_id: u64,
    #[serde(rename = "fileID")]
    pub file_id: u64,
    #[serde(default = "required_by_default")]
    pub required: bool,
}

fn manifest_type() -> String {
    "minecraftModpack".into()
}

fn manifest_version() -> u32 {
    1
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
/// automatically, from CurseForge or the Modrinth fallback, are returned so the user can fetch
/// them by hand.
pub async fn install_curseforge_pack(
    platforms: &ModPlatforms,
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
    let draft = DraftGuard::new(&instance);
    let blocked = populate(
        platforms,
        archive_path,
        &manifest,
        &instance,
        target.concurrency,
        on_progress,
    )
    .await?;
    instance.config.managed_pack = source.map(|source| ManagedPack {
        provider: source.provider,
        project_id: source.project_id,
        version_id: source.version_id,
        name: manifest.name.clone(),
        version: manifest.version.clone(),
    });
    instance.save()?;
    draft.keep();
    Ok(PackInstall { instance, blocked })
}

async fn populate(
    platforms: &ModPlatforms,
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
    let curseforge = &platforms.curseforge;
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
            let blocked_file = BlockedFile {
                project_name: project.name.clone(),
                file_name: file.file_name.clone(),
                url: format!("{}/files/{}", project.website_url, version.id),
                install_dir: project.install_dir.to_owned(),
            };
            blocked.push((blocked_file, file.sha1.clone()));
        } else {
            downloads.push(Download {
                path: game_dir.join(project.install_dir).join(&file.file_name),
                ..file.download_into(&game_dir)
            });
        }
    }
    let mut blocked = if platforms.modrinth_fallback_for_blocked {
        let (found, still_blocked) = modrinth_copies(platforms, blocked, &game_dir).await;
        downloads.extend(found);
        still_blocked
    } else {
        blocked.into_iter().map(|(file, _)| file).collect()
    };
    platforms
        .http()
        .download_all(downloads, concurrency, on_progress)
        .await?;
    let prefix = manifest.overrides.trim_matches('/').to_owned();
    archive::extract(archive_path, &game_dir, |name| {
        name.strip_prefix(&prefix).ok().map(PathBuf::from)
    })?;
    blocked.sort_by(|a, b| a.project_name.cmp(&b.project_name));
    Ok(blocked)
}

/// Looks blocked files up on Modrinth by SHA-1; returns downloads for the ones it has and the
/// files that stay blocked. A Modrinth failure only means nothing is found there.
async fn modrinth_copies(
    platforms: &ModPlatforms,
    blocked: Vec<(BlockedFile, Option<String>)>,
    game_dir: &Path,
) -> (Vec<Download>, Vec<BlockedFile>) {
    let hashes: Vec<String> = blocked
        .iter()
        .filter_map(|(_, sha1)| sha1.clone())
        .collect();
    let versions = match platforms.modrinth.versions_by_sha1(&hashes).await {
        Ok(versions) => versions,
        Err(err) => {
            log::warn!("Modrinth lookup for blocked CurseForge files failed: {err}");
            HashMap::new()
        }
    };
    let mut downloads = Vec::new();
    let mut still_blocked = Vec::new();
    for (file, sha1) in blocked {
        let copy = sha1.as_ref().and_then(|sha1| {
            versions
                .get(sha1)?
                .files
                .iter()
                .find(|candidate| candidate.sha1.as_ref() == Some(sha1))
        });
        match copy {
            Some(copy) => downloads.push(Download {
                url: copy.url.clone(),
                path: game_dir.join(&file.install_dir).join(&file.file_name),
                sha1: copy.sha1.clone(),
                size: Some(copy.size),
            }),
            None => still_blocked.push(file),
        }
    }
    (downloads, still_blocked)
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
