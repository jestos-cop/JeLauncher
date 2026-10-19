use std::collections::{HashSet, VecDeque};
use std::path::Path;

use serde::Deserialize;

use super::{
    Dependency, ProjectHit, ProjectVersion, Provider, ReleaseType, ResourceKind, SearchPage,
    SearchQuery, SortOrder, VersionFile,
};
use crate::instance::Instance;
use crate::minecraft::LoaderKind;
use crate::modpack::{PackSource, install_mrpack};
use crate::net::{DownloadProgress, HttpClient};
use crate::{Error, Result};

const API: &str = "https://api.modrinth.com/v2";

#[derive(Debug, Clone)]
pub struct Modrinth {
    http: HttpClient,
}

impl Modrinth {
    pub fn new(http: HttpClient) -> Self {
        Self { http }
    }

    pub async fn search(&self, query: &SearchQuery) -> Result<SearchPage> {
        let params = [
            ("query", query.text.clone()),
            ("facets", facets(query)),
            ("index", sort_index(query.sort).to_owned()),
            ("offset", query.offset.to_string()),
            ("limit", query.limit.clamp(1, 100).to_string()),
        ];
        let response: SearchResponse = self
            .http
            .get_json_query(&format!("{API}/search"), &params)
            .await?;
        Ok(SearchPage {
            total: response.total_hits,
            hits: response.hits.into_iter().map(Into::into).collect(),
        })
    }

    /// Versions compatible with the game version and, for mods, the loader; newest first.
    pub async fn versions(
        &self,
        project_id: &str,
        kind: ResourceKind,
        game_version: Option<&str>,
        loader: Option<LoaderKind>,
    ) -> Result<Vec<ProjectVersion>> {
        let mut params = Vec::new();
        if let Some(game_version) = game_version {
            params.push(("game_versions", json_list(game_version)));
        }
        if let (ResourceKind::Mod | ResourceKind::Modpack, Some(loader)) = (kind, loader) {
            params.push(("loaders", json_list(loader_name(loader))));
        }
        let url = format!("{API}/project/{project_id}/version");
        let versions: Vec<ApiVersion> = self.http.get_json_query(&url, &params).await?;
        versions.into_iter().map(TryInto::try_into).collect()
    }

    pub async fn version(&self, version_id: &str) -> Result<ProjectVersion> {
        let version: ApiVersion = self
            .http
            .get_json(&format!("{API}/version/{version_id}"))
            .await?;
        version.try_into()
    }

    /// Installs the version and, recursively, the required dependencies that fit the target.
    /// Returns the installed file names, the requested version first.
    pub async fn install_with_dependencies(
        &self,
        version: ProjectVersion,
        target: &InstallTarget<'_>,
    ) -> Result<Vec<String>> {
        let mut seen = HashSet::from([version.project_id.clone()]);
        let mut queue = VecDeque::from([version]);
        let mut installed = Vec::new();
        while let Some(version) = queue.pop_front() {
            installed.push(self.install(&version, target.dir).await?);
            for dependency in version.dependencies.iter().filter(|dep| dep.required) {
                let Some(project_id) = &dependency.project_id else {
                    continue;
                };
                if !seen.insert(project_id.clone()) {
                    continue;
                }
                queue.push_back(
                    self.resolve_dependency(project_id, dependency.version_id.as_deref(), target)
                        .await?,
                );
            }
        }
        Ok(installed)
    }

    async fn resolve_dependency(
        &self,
        project_id: &str,
        version_id: Option<&str>,
        target: &InstallTarget<'_>,
    ) -> Result<ProjectVersion> {
        if let Some(version_id) = version_id {
            return self.version(version_id).await;
        }
        self.versions(
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

    /// Downloads a modpack version and installs it as a new instance.
    pub async fn install_modpack(
        &self,
        version_id: &str,
        target: &ModpackTarget<'_>,
        on_progress: impl Fn(DownloadProgress),
    ) -> Result<Instance> {
        let version = self.version(version_id).await?;
        let archive_name = self.install(&version, target.download_dir).await?;
        let archive = target.download_dir.join(archive_name);
        let source = PackSource {
            provider: Provider::Modrinth,
            project_id: version.project_id,
            version_id: version.id,
        };
        let instance = install_mrpack(
            &self.http,
            &archive,
            target.instances_dir,
            target.name.clone(),
            Some(source),
            target.concurrency,
            on_progress,
        )
        .await;
        let _ = std::fs::remove_file(&archive);
        instance
    }

    /// Downloads the version's primary file into `dir` and returns the file name.
    pub async fn install(&self, version: &ProjectVersion, dir: &Path) -> Result<String> {
        let file = version
            .primary_file()
            .ok_or_else(|| Error::InvalidInput(format!("version {} has no files", version.name)))?;
        self.http.download(&file.download_into(dir)).await?;
        Ok(file.file_name.clone())
    }
}

/// Where a downloaded modpack becomes a new instance.
pub struct ModpackTarget<'a> {
    pub instances_dir: &'a Path,
    pub download_dir: &'a Path,
    pub name: Option<String>,
    pub concurrency: usize,
}

/// Where and for which game setup resources are installed.
pub struct InstallTarget<'a> {
    pub dir: &'a Path,
    pub kind: ResourceKind,
    pub game_version: &'a str,
    pub loader: Option<LoaderKind>,
}

fn facets(query: &SearchQuery) -> String {
    let mut groups = vec![vec![format!("project_type:{}", project_type(query.kind))]];
    if let Some(version) = &query.game_version {
        groups.push(vec![format!("versions:{version}")]);
    }
    if let (ResourceKind::Mod | ResourceKind::Modpack, Some(loader)) = (query.kind, query.loader) {
        groups.push(vec![format!("categories:{}", loader_name(loader))]);
    }
    serde_json::to_string(&groups).unwrap_or_default()
}

fn json_list(value: &str) -> String {
    serde_json::to_string(&[value]).unwrap_or_default()
}

fn project_type(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::Mod => "mod",
        ResourceKind::ResourcePack => "resourcepack",
        ResourceKind::ShaderPack => "shader",
        ResourceKind::DataPack => "datapack",
        ResourceKind::Modpack => "modpack",
    }
}

fn loader_name(loader: LoaderKind) -> &'static str {
    match loader {
        LoaderKind::Fabric => "fabric",
        LoaderKind::Quilt => "quilt",
    }
}

fn sort_index(sort: SortOrder) -> &'static str {
    match sort {
        SortOrder::Relevance => "relevance",
        SortOrder::Downloads => "downloads",
        SortOrder::Updated => "updated",
        SortOrder::Newest => "newest",
    }
}

#[derive(Deserialize)]
struct SearchResponse {
    hits: Vec<ApiHit>,
    total_hits: u32,
}

#[derive(Deserialize)]
struct ApiHit {
    project_id: String,
    slug: String,
    title: String,
    description: String,
    author: String,
    icon_url: Option<String>,
    downloads: u64,
}

impl From<ApiHit> for ProjectHit {
    fn from(hit: ApiHit) -> Self {
        Self {
            provider: Provider::Modrinth,
            id: hit.project_id,
            slug: hit.slug,
            title: hit.title,
            description: hit.description,
            author: hit.author,
            icon_url: hit.icon_url.filter(|url| !url.is_empty()),
            downloads: hit.downloads,
        }
    }
}

#[derive(Deserialize)]
struct ApiVersion {
    id: String,
    project_id: String,
    name: String,
    version_number: String,
    version_type: String,
    date_published: String,
    files: Vec<ApiFile>,
    #[serde(default)]
    dependencies: Vec<ApiDependency>,
}

#[derive(Deserialize)]
struct ApiFile {
    url: String,
    filename: String,
    hashes: ApiHashes,
    size: u64,
    primary: bool,
}

#[derive(Deserialize)]
struct ApiHashes {
    sha1: Option<String>,
}

#[derive(Deserialize)]
struct ApiDependency {
    project_id: Option<String>,
    version_id: Option<String>,
    dependency_type: String,
}

impl TryFrom<ApiVersion> for ProjectVersion {
    type Error = Error;

    fn try_from(version: ApiVersion) -> Result<Self> {
        let files = version
            .files
            .into_iter()
            .map(|file| {
                // The name comes from the network, so keep only the final path component.
                let file_name = Path::new(&file.filename)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .filter(|name| !name.is_empty())
                    .ok_or_else(|| {
                        Error::InvalidInput(format!("invalid file name {}", file.filename))
                    })?
                    .to_owned();
                Ok(VersionFile {
                    url: file.url,
                    file_name,
                    sha1: file.hashes.sha1,
                    size: file.size,
                    primary: file.primary,
                })
            })
            .collect::<Result<_>>()?;
        Ok(Self {
            id: version.id,
            project_id: version.project_id,
            name: version.name,
            version_number: version.version_number,
            release_type: match version.version_type.as_str() {
                "beta" => ReleaseType::Beta,
                "alpha" => ReleaseType::Alpha,
                _ => ReleaseType::Release,
            },
            published: version.date_published,
            files,
            dependencies: version
                .dependencies
                .into_iter()
                .filter(|dep| {
                    dep.dependency_type == "required" || dep.dependency_type == "optional"
                })
                .map(|dep| Dependency {
                    required: dep.dependency_type == "required",
                    project_id: dep.project_id,
                    version_id: dep.version_id,
                })
                .collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_facets_for_mods_with_loader() {
        let query = SearchQuery {
            text: "sodium".into(),
            kind: ResourceKind::Mod,
            game_version: Some("1.21.4".into()),
            loader: Some(LoaderKind::Fabric),
            sort: SortOrder::Relevance,
            offset: 0,
            limit: 20,
        };
        assert_eq!(
            facets(&query),
            r#"[["project_type:mod"],["versions:1.21.4"],["categories:fabric"]]"#
        );
        let packs = SearchQuery {
            kind: ResourceKind::ResourcePack,
            ..query
        };
        assert_eq!(
            facets(&packs),
            r#"[["project_type:resourcepack"],["versions:1.21.4"]]"#
        );
    }

    #[test]
    fn rejects_file_names_with_paths() -> std::result::Result<(), serde_json::Error> {
        let version: ApiVersion = serde_json::from_str(
            r#"{"id":"v","project_id":"p","name":"n","version_number":"1","version_type":"beta","date_published":"d",
                "files":[{"url":"u","filename":"../../evil.jar","hashes":{"sha1":"s"},"size":1,"primary":true}]}"#,
        )?;
        let converted = ProjectVersion::try_from(version);
        assert_eq!(
            converted.ok().map(|v| v.files[0].file_name.clone()),
            Some("evil.jar".into())
        );
        Ok(())
    }
}
