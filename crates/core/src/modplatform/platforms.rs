use super::atlauncher::AtLauncher;
use super::curseforge::CurseForge;
use super::ftb::Ftb;
use super::modrinth::Modrinth;
use super::technic::Technic;
use super::{ProjectVersion, Provider, ResourceKind, SearchPage, SearchQuery, download_version};
use crate::minecraft::LoaderKind;
use crate::modpack::{
    ModpackTarget, PackInstall, PackSource, install_atlauncher_pack, install_curseforge_pack,
    install_ftb_pack, install_technic_pack,
};
use crate::net::{DownloadProgress, HttpClient};
use crate::{Error, Result};

/// Every supported mod platform behind one interface, chosen per call by [`Provider`].
#[derive(Debug, Clone)]
pub struct ModPlatforms {
    pub modrinth: Modrinth,
    pub curseforge: CurseForge,
    pub ftb: Ftb,
    pub technic: Technic,
    pub atlauncher: AtLauncher,
    pub modrinth_fallback_for_blocked: bool,
    http: HttpClient,
}

/// Credentials and behaviour of the platforms; every credential is optional.
#[derive(Debug, Clone, Default)]
pub struct PlatformConfig {
    pub curseforge_api_key: Option<String>,
    pub modrinth_token: Option<String>,
    pub technic_client_id: Option<String>,
    /// Downloads CurseForge files their authors block from Modrinth when it has the same file.
    pub modrinth_fallback_for_blocked: bool,
}

impl ModPlatforms {
    pub fn new(http: HttpClient, config: PlatformConfig) -> Self {
        Self {
            modrinth: Modrinth::with_token(http.clone(), config.modrinth_token),
            curseforge: CurseForge::new(http.clone(), config.curseforge_api_key),
            ftb: Ftb::new(http.clone()),
            technic: Technic::with_client_id(http.clone(), config.technic_client_id),
            modrinth_fallback_for_blocked: config.modrinth_fallback_for_blocked,
            atlauncher: AtLauncher::new(http.clone()),
            http,
        }
    }

    pub fn http(&self) -> &HttpClient {
        &self.http
    }

    /// Downloads a modpack version and installs it as a new instance, recording where its
    /// files came from when the target asks for it.
    pub async fn install_modpack(
        &self,
        provider: Provider,
        project_id: &str,
        version_id: &str,
        target: &ModpackTarget<'_>,
        on_progress: impl Fn(DownloadProgress),
    ) -> Result<PackInstall> {
        let installed = self
            .install_pack_files(provider, project_id, version_id, target, on_progress)
            .await?;
        if target.track_sources {
            self.identify_pack_sources(&installed.instance, target.source_order)
                .await;
        }
        Ok(installed)
    }

    async fn install_pack_files(
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
                let installed =
                    install_curseforge_pack(self, &archive, target, Some(source), on_progress)
                        .await;
                let _ = std::fs::remove_file(&archive);
                installed
            }
            Provider::Ftb => {
                install_ftb_pack(
                    &self.ftb,
                    &self.http,
                    project_id,
                    version_id,
                    target,
                    on_progress,
                )
                .await
            }
            Provider::Technic => {
                install_technic_pack(
                    &self.technic,
                    &self.http,
                    project_id,
                    version_id,
                    target,
                    on_progress,
                )
                .await
            }
            Provider::AtLauncher => {
                install_atlauncher_pack(
                    &self.atlauncher,
                    &self.http,
                    project_id,
                    version_id,
                    target,
                    on_progress,
                )
                .await
            }
        }
    }

    pub async fn search(&self, query: &SearchQuery) -> Result<SearchPage> {
        match query.provider {
            Provider::Modrinth => self.modrinth.search(query).await,
            Provider::CurseForge => self.curseforge.search(query).await,
            Provider::Ftb if query.kind == ResourceKind::Modpack => self.ftb.search(query).await,
            Provider::Technic if query.kind == ResourceKind::Modpack => {
                self.technic.search(query).await
            }
            Provider::AtLauncher if query.kind == ResourceKind::Modpack => {
                self.atlauncher.search(query).await
            }
            Provider::Ftb | Provider::Technic | Provider::AtLauncher => {
                Err(packs_only(query.provider))
            }
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
            Provider::Ftb if kind == ResourceKind::Modpack => self.ftb.versions(project_id).await,
            Provider::Technic if kind == ResourceKind::Modpack => {
                self.technic.versions(project_id).await
            }
            Provider::AtLauncher if kind == ResourceKind::Modpack => {
                self.atlauncher.versions(project_id).await
            }
            Provider::Ftb | Provider::Technic | Provider::AtLauncher => Err(packs_only(provider)),
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
            Provider::Ftb => self.ftb.version(project_id, version_id).await,
            Provider::Technic => self.technic.version(project_id, version_id).await,
            Provider::AtLauncher => self.atlauncher.version(project_id, version_id).await,
        }
    }
}

fn packs_only(provider: Provider) -> Error {
    Error::InvalidInput(format!("{provider:?} publishes modpacks only"))
}
