use std::collections::{HashSet, VecDeque};

use super::curseforge::CurseForge;
use super::modrinth::Modrinth;
use super::{
    InstallTarget, ProjectVersion, Provider, ResourceKind, SearchPage, SearchQuery,
    download_version,
};
use crate::minecraft::LoaderKind;
use crate::modpack::{ModpackTarget, PackInstall, PackSource, install_curseforge_pack};
use crate::net::{DownloadProgress, HttpClient};
use crate::{Error, Result};

/// Every supported mod platform behind one interface, chosen per call by [`Provider`].
#[derive(Debug, Clone)]
pub struct ModPlatforms {
    pub modrinth: Modrinth,
    pub curseforge: CurseForge,
    http: HttpClient,
}

impl ModPlatforms {
    pub fn new(http: HttpClient, curseforge_api_key: Option<String>) -> Self {
        Self {
            modrinth: Modrinth::new(http.clone()),
            curseforge: CurseForge::new(http.clone(), curseforge_api_key),
            http,
        }
    }

    pub fn http(&self) -> &HttpClient {
        &self.http
    }

    /// Downloads a modpack version and installs it as a new instance.
    pub async fn install_modpack(
        &self,
        provider: Provider,
        project_id: &str,
        version_id: &str,
        target: &ModpackTarget<'_>,
        on_progress: impl Fn(DownloadProgress),
    ) -> Result<PackInstall> {
        match provider {
            Provider::Modrinth => self
                .modrinth
                .install_modpack(version_id, target, on_progress)
                .await
                .map(PackInstall::complete),
            Provider::CurseForge => {
                let version = self.curseforge.version(project_id, version_id).await?;
                let archive_name =
                    download_version(&self.http, &version, target.download_dir).await?;
                let archive = target.download_dir.join(archive_name);
                let source = PackSource {
                    provider,
                    project_id: version.project_id,
                    version_id: version.id,
                };
                let installed = install_curseforge_pack(
                    &self.curseforge,
                    &self.http,
                    &archive,
                    target,
                    Some(source),
                    on_progress,
                )
                .await;
                let _ = std::fs::remove_file(&archive);
                installed
            }
        }
    }

    pub async fn search(&self, query: &SearchQuery) -> Result<SearchPage> {
        match query.provider {
            Provider::Modrinth => self.modrinth.search(query).await,
            Provider::CurseForge => self.curseforge.search(query).await,
        }
    }

    pub async fn versions(
        &self,
        provider: Provider,
        project_id: &str,
        kind: ResourceKind,
        game_version: Option<&str>,
        loader: Option<LoaderKind>,
    ) -> Result<Vec<ProjectVersion>> {
        match provider {
            Provider::Modrinth => {
                self.modrinth
                    .versions(project_id, kind, game_version, loader)
                    .await
            }
            Provider::CurseForge => {
                self.curseforge
                    .versions(project_id, kind, game_version, loader)
                    .await
            }
        }
    }

    pub async fn version(
        &self,
        provider: Provider,
        project_id: &str,
        version_id: &str,
    ) -> Result<ProjectVersion> {
        match provider {
            Provider::Modrinth => self.modrinth.version(version_id).await,
            Provider::CurseForge => self.curseforge.version(project_id, version_id).await,
        }
    }

    /// Installs the version and, recursively, the required dependencies that fit the target.
    /// Returns the installed file names, the requested version first.
    pub async fn install_with_dependencies(
        &self,
        provider: Provider,
        version: ProjectVersion,
        target: &InstallTarget<'_>,
    ) -> Result<Vec<String>> {
        let mut seen = HashSet::from([version.project_id.clone()]);
        let mut queue = VecDeque::from([version]);
        let mut installed = Vec::new();
        while let Some(version) = queue.pop_front() {
            installed.push(download_version(&self.http, &version, target.dir).await?);
            for dependency in version.dependencies.iter().filter(|dep| dep.required) {
                let Some(project_id) = &dependency.project_id else {
                    continue;
                };
                if !seen.insert(project_id.clone()) {
                    continue;
                }
                let resolved = match &dependency.version_id {
                    Some(version_id) => self.version(provider, project_id, version_id).await?,
                    None => self.latest_compatible(provider, project_id, target).await?,
                };
                queue.push_back(resolved);
            }
        }
        Ok(installed)
    }

    async fn latest_compatible(
        &self,
        provider: Provider,
        project_id: &str,
        target: &InstallTarget<'_>,
    ) -> Result<ProjectVersion> {
        self.versions(
            provider,
            project_id,
            target.kind,
            Some(target.game_version),
            target.loader,
        )
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| {
            Error::InvalidInput(format!(
                "no version of required dependency {project_id} supports Minecraft {}",
                target.game_version
            ))
        })
    }
}
