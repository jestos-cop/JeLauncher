use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::{
    Dependency, InstallTarget, ProjectHit, ProjectVersion, Provider, ReleaseType, ResourceKind,
    ResourceUpdate, SearchPage, SearchQuery, SortOrder, VersionFile, download_version,
};
use crate::instance::Instance;
use crate::minecraft::LoaderKind;
use crate::modpack::{ModpackTarget, PackSource, install_mrpack, update_mrpack};
use crate::net::{DownloadProgress, HttpClient, sha1_of};
use crate::resources::{DISABLED_SUFFIX, list_resources, set_resource_enabled};
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
            params.push(("loaders", json_list(loader.id())));
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

    /// Looks up the published versions that contain files with the given SHA-1 hashes.
    pub async fn versions_by_sha1(
        &self,
        hashes: &[String],
    ) -> Result<HashMap<String, ProjectVersion>> {
        if hashes.is_empty() {
            return Ok(HashMap::new());
        }
        let body = HashLookup {
            hashes,
            algorithm: "sha1",
        };
        let versions: HashMap<String, ApiVersion> = self
            .http
            .post_json(&format!("{API}/version_files"), &body)
            .await?;
        versions
            .into_iter()
            .map(|(hash, version)| Ok((hash, version.try_into()?)))
            .collect()
    }

    /// Finds newer versions of the resource files in `target.dir` that fit the target.
    pub async fn check_updates(&self, target: &InstallTarget<'_>) -> Result<Vec<ResourceUpdate>> {
        let mut by_hash = HashMap::new();
        for file in list_resources(target.dir, target.kind)? {
            let path = target.dir.join(&file.file_name);
            if path.is_file() {
                by_hash.insert(sha1_of(&path).await?, file.file_name);
            }
        }
        if by_hash.is_empty() {
            return Ok(Vec::new());
        }
        let hashes: Vec<String> = by_hash.keys().cloned().collect();
        let body = UpdateLookup {
            hashes: &hashes,
            algorithm: "sha1",
            loaders: match (target.kind, target.loader) {
                (ResourceKind::Mod, Some(loader)) => vec![loader.id()],
                _ => Vec::new(),
            },
            game_versions: vec![target.game_version],
        };
        let latest: HashMap<String, ApiVersion> = self
            .http
            .post_json(&format!("{API}/version_files/update"), &body)
            .await?;
        let mut updates = Vec::new();
        for (hash, version) in latest {
            let version = ProjectVersion::try_from(version)?;
            let is_current = version
                .files
                .iter()
                .any(|file| file.sha1.as_deref() == Some(hash.as_str()));
            if let (false, Some(file_name)) = (is_current, by_hash.remove(&hash)) {
                updates.push(ResourceUpdate { file_name, version });
            }
        }
        updates.sort_by(|a, b| a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase()));
        Ok(updates)
    }

    /// Replaces an installed file with the primary file of `version`, keeping it disabled if it was.
    /// Returns the new file name.
    pub async fn apply_update(
        &self,
        dir: &Path,
        file_name: &str,
        version: &ProjectVersion,
    ) -> Result<String> {
        let old = crate::fs::child_path(dir, file_name)?;
        let new_name = self.install(version, dir).await?;
        let was_disabled = file_name.ends_with(DISABLED_SUFFIX);
        if new_name != file_name {
            std::fs::remove_file(&old).map_err(|err| Error::io(&old, err))?;
        }
        if was_disabled {
            return set_resource_enabled(dir, &new_name, false);
        }
        Ok(new_name)
    }

    /// Downloads a modpack version and installs it as a new instance.
    pub async fn install_modpack(
        &self,
        version_id: &str,
        target: &ModpackTarget<'_>,
        on_progress: impl Fn(DownloadProgress),
    ) -> Result<Instance> {
        let (archive, source) = self
            .download_modpack(version_id, target.download_dir)
            .await?;
        let instance =
            install_mrpack(&self.http, &archive, target, Some(source), on_progress).await;
        let _ = std::fs::remove_file(&archive);
        instance
    }

    /// Switches a modpack instance to another version of the same pack.
    pub async fn update_modpack(
        &self,
        instance: &mut Instance,
        version_id: &str,
        download_dir: &Path,
        concurrency: usize,
        on_progress: impl Fn(DownloadProgress),
    ) -> Result<()> {
        let (archive, source) = self.download_modpack(version_id, download_dir).await?;
        let updated = update_mrpack(
            &self.http,
            &archive,
            instance,
            source,
            concurrency,
            on_progress,
        )
        .await;
        let _ = std::fs::remove_file(&archive);
        updated
    }

    async fn download_modpack(
        &self,
        version_id: &str,
        download_dir: &Path,
    ) -> Result<(std::path::PathBuf, PackSource)> {
        let version = self.version(version_id).await?;
        let archive_name = self.install(&version, download_dir).await?;
        let source = PackSource {
            provider: Provider::Modrinth,
            project_id: version.project_id,
            version_id: version.id,
        };
        Ok((download_dir.join(archive_name), source))
    }

    /// Downloads the version's primary file into `dir` and returns the file name.
    pub async fn install(&self, version: &ProjectVersion, dir: &Path) -> Result<String> {
        download_version(&self.http, version, dir).await
    }
}

fn facets(query: &SearchQuery) -> String {
    let mut groups = vec![vec![format!("project_type:{}", project_type(query.kind))]];
    if let Some(version) = &query.game_version {
        groups.push(vec![format!("versions:{version}")]);
    }
    if let (ResourceKind::Mod | ResourceKind::Modpack, Some(loader)) = (query.kind, query.loader) {
        groups.push(vec![format!("categories:{}", loader.id())]);
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

fn sort_index(sort: SortOrder) -> &'static str {
    match sort {
        SortOrder::Relevance => "relevance",
        SortOrder::Downloads => "downloads",
        SortOrder::Updated => "updated",
        SortOrder::Newest => "newest",
    }
}

#[derive(Serialize)]
struct UpdateLookup<'a> {
    hashes: &'a [String],
    algorithm: &'a str,
    loaders: Vec<&'a str>,
    game_versions: Vec<&'a str>,
}

#[derive(Serialize)]
struct HashLookup<'a> {
    hashes: &'a [String],
    algorithm: &'a str,
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
    #[serde(default)]
    changelog: Option<String>,
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
    sha512: Option<String>,
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
                    sha512: file.hashes.sha512,
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
            changelog: version
                .changelog
                .filter(|changelog| !changelog.trim().is_empty()),
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
            provider: Provider::Modrinth,
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
