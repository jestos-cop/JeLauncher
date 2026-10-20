//! Newer versions of installed resources, checked on the platform each file came from.

use futures_util::{StreamExt, TryStreamExt, stream};

use super::{
    InstallTarget, InstalledFile, ModPlatforms, Provider, ResourceUpdate, replace_resource,
};
use crate::Result;
use crate::resources::{ResourceFile, ResourceSource, attach_sources, list_resources};
use crate::settings::UpdateChannel;

/// CurseForge has no batch update endpoint, so projects are asked a few at a time.
const PARALLEL_REQUESTS: usize = 4;

impl ModPlatforms {
    /// Newer versions of the resources in `target.dir`. Files with a CurseForge source are
    /// checked on CurseForge, all others on Modrinth by hash; locked files are skipped.
    pub async fn check_updates(
        &self,
        target: &InstallTarget<'_>,
        channel: UpdateChannel,
    ) -> Result<Vec<ResourceUpdate>> {
        let mut files = list_resources(target.dir, target.kind)?;
        if target.track_sources {
            attach_sources(target.dir, target.kind, &mut files)?;
        }
        let candidates: Vec<ResourceFile> = files
            .into_iter()
            .filter(|file| {
                !file
                    .source
                    .as_ref()
                    .is_some_and(|source| source.lock_update)
            })
            .collect();
        let (curseforge, modrinth): (Vec<ResourceFile>, Vec<ResourceFile>) =
            candidates.into_iter().partition(|file| {
                file.source
                    .as_ref()
                    .is_some_and(|source| source.provider == Provider::CurseForge)
            });

        let modrinth_names: Vec<String> = modrinth.into_iter().map(|file| file.file_name).collect();
        let mut updates = self
            .modrinth
            .check_updates(target, &modrinth_names, channel)
            .await?;
        let curseforge_updates: Vec<Option<ResourceUpdate>> = stream::iter(
            curseforge
                .into_iter()
                .filter_map(|file| Some((file.file_name, file.source?)))
                .map(|(file_name, source)| {
                    self.curseforge_update(target, file_name, source, channel)
                }),
        )
        .buffered(PARALLEL_REQUESTS)
        .try_collect()
        .await?;
        updates.extend(curseforge_updates.into_iter().flatten());
        updates.sort_by_key(|update| update.file_name.to_lowercase());
        Ok(updates)
    }

    /// Replaces an installed file with another version of its project and records the new
    /// source. Returns the new file name.
    pub async fn update_resource(
        &self,
        target: &InstallTarget<'_>,
        file_name: &str,
        provider: Provider,
        project_id: &str,
        version_id: &str,
    ) -> Result<String> {
        let version = self.version(provider, project_id, version_id).await?;
        let new_name = replace_resource(self.http(), target.dir, file_name, &version).await?;
        if let (true, Some(file)) = (target.track_sources, version.primary_file()) {
            let installed = InstalledFile {
                file_name: new_name.clone(),
                file: file.clone(),
                version: version.clone(),
            };
            self.record_sources(target.dir, target.kind, provider, &[installed], false)
                .await?;
        }
        Ok(new_name)
    }

    async fn curseforge_update(
        &self,
        target: &InstallTarget<'_>,
        file_name: String,
        source: ResourceSource,
        channel: UpdateChannel,
    ) -> Result<Option<ResourceUpdate>> {
        let mut versions = self
            .curseforge
            .versions(
                &source.project_id,
                target.kind,
                Some(target.game_version),
                target.loader,
            )
            .await?;
        versions.sort_by(|a, b| b.published.cmp(&a.published));
        let latest = versions
            .into_iter()
            .find(|version| channel.allows(version.release_type));
        Ok(latest
            .filter(|version| version.id != source.version_id)
            .map(|version| ResourceUpdate {
                file_name,
                provider: Provider::CurseForge,
                version,
            }))
    }
}
