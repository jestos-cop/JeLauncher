//! Recording and recovering which platform project installed resource files came from.

use std::collections::HashMap;
use std::path::Path;

use serde::Serialize;

use super::curseforge::fingerprint;
use super::{ModPlatforms, ProjectVersion, Provider, VersionFile};
use crate::instance::Instance;
use crate::net::sha1_of;
use crate::resources::{
    ProjectInfo, ResourceKind, ResourceSource, attach_sources, list_resources, write_source,
};
use crate::{Error, Result};

/// Kinds of resources that modpacks ship and platforms publish.
const PACK_KINDS: [ResourceKind; 3] = [
    ResourceKind::Mod,
    ResourceKind::ResourcePack,
    ResourceKind::ShaderPack,
];

/// A file installed from a platform version.
pub struct InstalledFile {
    pub file_name: String,
    pub version: ProjectVersion,
    pub file: VersionFile,
}

/// How many files without a source a lookup found and how many stay unknown.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct IdentifyReport {
    pub identified: u32,
    pub unknown: u32,
}

impl ModPlatforms {
    /// Slugs and names of projects on `provider`, keyed by project id.
    pub async fn project_infos(
        &self,
        provider: Provider,
        ids: &[String],
    ) -> Result<HashMap<String, ProjectInfo>> {
        match provider {
            Provider::Modrinth => self.modrinth.projects(ids).await,
            Provider::CurseForge => {
                let ids: Vec<u64> = ids.iter().filter_map(|id| id.parse().ok()).collect();
                Ok(self
                    .curseforge
                    .projects(&ids)
                    .await?
                    .into_iter()
                    .map(|project| {
                        let info = ProjectInfo {
                            slug: project.slug,
                            name: project.name,
                        };
                        (project.id, info)
                    })
                    .collect())
            }
            Provider::Ftb | Provider::Technic | Provider::AtLauncher => Ok(HashMap::new()),
        }
    }

    /// Writes the sources of files just installed from `provider` into the index of `dir`.
    pub async fn record_sources(
        &self,
        dir: &Path,
        kind: ResourceKind,
        provider: Provider,
        installed: &[InstalledFile],
        lock_update: bool,
    ) -> Result<()> {
        let mut ids: Vec<String> = installed
            .iter()
            .map(|file| file.version.project_id.clone())
            .collect();
        ids.sort();
        ids.dedup();
        let infos = self.project_infos(provider, &ids).await?;
        for installed in installed {
            if let Some(project) = infos.get(&installed.version.project_id) {
                let source = ResourceSource::new(
                    provider,
                    project,
                    &installed.version,
                    &installed.file,
                    &installed.file_name,
                    lock_update,
                );
                write_source(dir, kind, &source)?;
            }
        }
        Ok(())
    }

    /// Finds the sources of files in `dir` that have none: on Modrinth by SHA-1, then on
    /// CurseForge by fingerprint. A platform that cannot be reached only leaves files unknown.
    pub async fn identify_sources(
        &self,
        dir: &Path,
        kind: ResourceKind,
        lock_update: bool,
    ) -> Result<IdentifyReport> {
        let mut files = list_resources(dir, kind)?;
        attach_sources(dir, kind, &mut files)?;
        let mut unknown: Vec<String> = files
            .into_iter()
            .filter(|file| file.source.is_none() && dir.join(&file.file_name).is_file())
            .map(|file| file.file_name)
            .collect();
        let total = unknown.len();
        if total == 0 {
            return Ok(IdentifyReport::default());
        }

        let found = self
            .find_on_modrinth(dir, &unknown)
            .await
            .unwrap_or_else(|err| {
                log::warn!("Modrinth source lookup failed: {err}");
                Vec::new()
            });
        unknown.retain(|name| !found.iter().any(|file| &file.file_name == name));
        self.record_sources(dir, kind, Provider::Modrinth, &found, lock_update)
            .await?;
        let mut identified = found.len();

        if !unknown.is_empty() {
            let found = self
                .find_on_curseforge(dir, &unknown)
                .await
                .unwrap_or_else(|err| {
                    log::warn!("CurseForge source lookup failed: {err}");
                    Vec::new()
                });
            self.record_sources(dir, kind, Provider::CurseForge, &found, lock_update)
                .await?;
            identified += found.len();
        }
        Ok(IdentifyReport {
            identified: u32::try_from(identified).unwrap_or(u32::MAX),
            unknown: u32::try_from(total - identified).unwrap_or(u32::MAX),
        })
    }

    /// Identifies the mods and packs a modpack installed; their updates belong to the pack.
    pub async fn identify_pack_sources(&self, instance: &Instance) {
        for kind in PACK_KINDS {
            if let Err(err) = self
                .identify_sources(&instance.resource_dir(kind), kind, true)
                .await
            {
                log::warn!(
                    "could not identify {kind:?} sources of {}: {err}",
                    instance.id
                );
            }
        }
    }

    async fn find_on_modrinth(&self, dir: &Path, names: &[String]) -> Result<Vec<InstalledFile>> {
        let mut by_hash = HashMap::new();
        for name in names {
            by_hash.insert(sha1_of(&dir.join(name)).await?, name.clone());
        }
        let hashes: Vec<String> = by_hash.keys().cloned().collect();
        let versions = self.modrinth.versions_by_sha1(&hashes).await?;
        Ok(versions
            .into_iter()
            .filter_map(|(hash, version)| {
                let file = version
                    .files
                    .iter()
                    .find(|file| file.sha1.as_deref() == Some(hash.as_str()))?
                    .clone();
                Some(InstalledFile {
                    file_name: by_hash.remove(&hash)?,
                    version,
                    file,
                })
            })
            .collect())
    }

    async fn find_on_curseforge(&self, dir: &Path, names: &[String]) -> Result<Vec<InstalledFile>> {
        let mut by_fingerprint = HashMap::new();
        for name in names {
            let path = dir.join(name);
            let bytes = tokio::fs::read(&path)
                .await
                .map_err(|err| Error::io(&path, err))?;
            by_fingerprint.insert(fingerprint(&bytes), name.clone());
        }
        let fingerprints: Vec<u32> = by_fingerprint.keys().copied().collect();
        let matches = self.curseforge.match_fingerprints(&fingerprints).await?;
        let file_ids: Vec<u64> = matches.values().map(|(_, file_id)| *file_id).collect();
        let versions: HashMap<String, ProjectVersion> = self
            .curseforge
            .files(&file_ids)
            .await?
            .into_iter()
            .map(|version| (version.id.clone(), version))
            .collect();
        Ok(matches
            .into_iter()
            .filter_map(|(fingerprint, (_, file_id))| {
                let version = versions.get(&file_id.to_string())?.clone();
                let file = version.primary_file()?.clone();
                Some(InstalledFile {
                    file_name: by_fingerprint.remove(&fingerprint)?,
                    version,
                    file,
                })
            })
            .collect())
    }
}
