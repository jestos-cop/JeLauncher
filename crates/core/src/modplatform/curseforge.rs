use std::collections::HashMap;

use serde::Deserialize;

use super::{
    Dependency, DependencyKind, ProjectHit, ProjectVersion, Provider, ReleaseType, ResourceKind,
    SearchPage, SearchQuery, SortOrder, VersionFile,
};
use crate::minecraft::LoaderKind;
use crate::net::HttpClient;
use crate::{Error, Result};

const API: &str = "https://api.curseforge.com/v1";
const MINECRAFT_GAME_ID: &str = "432";
/// The API rejects requests that page past this many results.
const MAX_RESULTS: u32 = 10_000;
const SHA1_ALGORITHM: u32 = 1;
const EMBEDDED_DEPENDENCY: u32 = 1;
const OPTIONAL_DEPENDENCY: u32 = 2;
const REQUIRED_DEPENDENCY: u32 = 3;
const INCOMPATIBLE_DEPENDENCY: u32 = 5;

#[derive(Debug, Clone)]
pub struct CurseForge {
    http: HttpClient,
    api_key: Option<String>,
}

impl CurseForge {
    /// Without an API key every request fails with a clear error.
    pub fn new(http: HttpClient, api_key: Option<String>) -> Self {
        Self {
            http,
            api_key: api_key.filter(|key| !key.trim().is_empty()),
        }
    }

    pub async fn search(&self, query: &SearchQuery) -> Result<SearchPage> {
        let mut params = vec![
            ("gameId", MINECRAFT_GAME_ID.to_owned()),
            ("classId", class_id(query.kind).to_string()),
            ("searchFilter", query.text.clone()),
            ("sortField", sort_field(query.sort).to_string()),
            ("sortOrder", "desc".to_owned()),
            ("index", query.offset.min(MAX_RESULTS - 1).to_string()),
            ("pageSize", query.limit.clamp(1, 50).to_string()),
        ];
        if let Some(version) = &query.game_version {
            params.push(("gameVersion", version.clone()));
        }
        if let (ResourceKind::Mod | ResourceKind::Modpack, Some(loader)) =
            (query.kind, query.loader)
        {
            params.push(("modLoaderType", loader_type(loader).to_string()));
        }
        let response: Paged<ApiMod> = self.get("/mods/search", &params).await?;
        Ok(SearchPage {
            total: response.pagination.total_count.min(MAX_RESULTS),
            hits: response.data.into_iter().map(Into::into).collect(),
        })
    }

    /// Files compatible with the game version and, for mods, the loader; newest first.
    pub async fn versions(
        &self,
        project_id: &str,
        kind: ResourceKind,
        game_version: Option<&str>,
        loader: Option<LoaderKind>,
    ) -> Result<Vec<ProjectVersion>> {
        let mut params = vec![("pageSize", "50".to_owned())];
        if let Some(version) = game_version {
            params.push(("gameVersion", version.to_owned()));
        }
        if let (ResourceKind::Mod | ResourceKind::Modpack, Some(loader)) = (kind, loader) {
            params.push(("modLoaderType", loader_type(loader).to_string()));
        }
        let response: Paged<ApiFile> = self
            .get(&format!("/mods/{project_id}/files"), &params)
            .await?;
        response.data.into_iter().map(TryInto::try_into).collect()
    }

    pub async fn version(&self, project_id: &str, version_id: &str) -> Result<ProjectVersion> {
        let response: Single<ApiFile> = self
            .get(&format!("/mods/{project_id}/files/{version_id}"), &[])
            .await?;
        response.data.try_into()
    }

    /// Several files at once, e.g. everything a modpack manifest lists.
    pub async fn files(&self, file_ids: &[u64]) -> Result<Vec<ProjectVersion>> {
        if file_ids.is_empty() {
            return Ok(Vec::new());
        }
        let body = serde_json::json!({ "fileIds": file_ids });
        let response: Single<Vec<ApiFile>> = self.post("/mods/files", &body).await?;
        response.data.into_iter().map(TryInto::try_into).collect()
    }

    pub async fn projects(&self, mod_ids: &[u64]) -> Result<Vec<CurseProject>> {
        if mod_ids.is_empty() {
            return Ok(Vec::new());
        }
        let body = serde_json::json!({ "modIds": mod_ids });
        let response: Single<Vec<ApiProject>> = self.post("/mods", &body).await?;
        Ok(response.data.into_iter().map(Into::into).collect())
    }

    /// Published files whose fingerprint matches, as `fingerprint -> (project id, file id)`.
    pub async fn match_fingerprints(
        &self,
        fingerprints: &[u32],
    ) -> Result<HashMap<u32, (u64, u64)>> {
        if fingerprints.is_empty() {
            return Ok(HashMap::new());
        }
        let body = serde_json::json!({ "fingerprints": fingerprints });
        let response: Single<FingerprintMatches> = self
            .post(&format!("/fingerprints/{MINECRAFT_GAME_ID}"), &body)
            .await?;
        Ok(response
            .data
            .exact_matches
            .into_iter()
            .map(|found| (found.file.file_fingerprint, (found.id, found.file.id)))
            .collect())
    }

    fn key(&self) -> Result<&str> {
        self.api_key.as_deref().ok_or_else(|| {
            Error::InvalidInput("CurseForge is not available: no API key is configured".into())
        })
    }

    async fn get<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        params: &[(&str, String)],
    ) -> Result<T> {
        self.http
            .get_json_with_header(&format!("{API}{path}"), params, ("x-api-key", self.key()?))
            .await
    }

    async fn post<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> Result<T> {
        self.http
            .post_json_with_header(&format!("{API}{path}"), body, ("x-api-key", self.key()?))
            .await
    }
}

/// Project details needed to place and attribute modpack files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurseProject {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub website_url: String,
    /// Where files of this project go inside the game directory.
    pub install_dir: &'static str,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiProject {
    id: u64,
    #[serde(default)]
    slug: String,
    name: String,
    class_id: Option<u32>,
    links: ApiLinks,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiLinks {
    #[serde(default)]
    website_url: String,
}

impl From<ApiProject> for CurseProject {
    fn from(project: ApiProject) -> Self {
        let kind = match project.class_id {
            Some(12) => ResourceKind::ResourcePack,
            Some(6552) => ResourceKind::ShaderPack,
            _ => ResourceKind::Mod,
        };
        Self {
            id: project.id.to_string(),
            slug: project.slug,
            name: project.name,
            website_url: project.links.website_url,
            install_dir: kind.install_dir(),
        }
    }
}

fn class_id(kind: ResourceKind) -> u32 {
    match kind {
        ResourceKind::Mod => 6,
        ResourceKind::ResourcePack => 12,
        ResourceKind::ShaderPack => 6552,
        ResourceKind::DataPack => 6945,
        ResourceKind::Modpack => 4471,
    }
}

fn loader_type(loader: LoaderKind) -> u32 {
    match loader {
        LoaderKind::Forge => 1,
        LoaderKind::Fabric => 4,
        LoaderKind::Quilt => 5,
        LoaderKind::NeoForge => 6,
    }
}

fn sort_field(sort: SortOrder) -> u32 {
    match sort {
        SortOrder::Relevance => 1,
        SortOrder::Updated => 3,
        SortOrder::Downloads => 6,
        SortOrder::Newest => 11,
    }
}

#[derive(Deserialize)]
struct Paged<T> {
    data: Vec<T>,
    pagination: Pagination,
}

#[derive(Deserialize)]
struct Single<T> {
    data: T,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Pagination {
    total_count: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiMod {
    id: u64,
    name: String,
    slug: String,
    summary: String,
    download_count: u64,
    #[serde(default)]
    authors: Vec<ApiAuthor>,
    logo: Option<ApiLogo>,
}

#[derive(Deserialize)]
struct ApiAuthor {
    name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiLogo {
    thumbnail_url: String,
}

impl From<ApiMod> for ProjectHit {
    fn from(project: ApiMod) -> Self {
        Self {
            provider: Provider::CurseForge,
            id: project.id.to_string(),
            slug: project.slug,
            title: project.name,
            description: project.summary,
            author: project
                .authors
                .into_iter()
                .next()
                .map(|author| author.name)
                .unwrap_or_default(),
            icon_url: project
                .logo
                .map(|logo| logo.thumbnail_url)
                .filter(|url| !url.is_empty()),
            downloads: project.download_count,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiFile {
    id: u64,
    mod_id: u64,
    display_name: String,
    file_name: String,
    release_type: u32,
    file_date: String,
    /// Missing when the author does not allow downloads through other launchers.
    download_url: Option<String>,
    file_length: u64,
    #[serde(default)]
    hashes: Vec<ApiHash>,
    #[serde(default)]
    dependencies: Vec<ApiDependency>,
    /// Game versions mixed with loader names such as `NeoForge` and sides such as `Client`.
    #[serde(default)]
    game_versions: Vec<String>,
}

#[derive(Deserialize)]
struct ApiHash {
    value: String,
    algo: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiDependency {
    mod_id: u64,
    relation_type: u32,
}

impl TryFrom<ApiFile> for ProjectVersion {
    type Error = Error;

    fn try_from(file: ApiFile) -> Result<Self> {
        let file_name = std::path::Path::new(&file.file_name)
            .file_name()
            .and_then(|name| name.to_str())
            .filter(|name| !name.is_empty())
            .ok_or_else(|| Error::InvalidInput(format!("invalid file name {}", file.file_name)))?
            .to_owned();
        Ok(Self {
            id: file.id.to_string(),
            project_id: file.mod_id.to_string(),
            name: file.file_name.clone(),
            version_number: file.display_name,
            release_type: match file.release_type {
                2 => ReleaseType::Beta,
                3 => ReleaseType::Alpha,
                _ => ReleaseType::Release,
            },
            published: file.file_date,
            changelog: None,
            files: vec![VersionFile {
                url: file.download_url.unwrap_or_default(),
                file_name,
                sha1: file
                    .hashes
                    .into_iter()
                    .find(|hash| hash.algo == SHA1_ALGORITHM)
                    .map(|hash| hash.value),
                sha512: None,
                size: file.file_length,
                primary: true,
            }],
            dependencies: file
                .dependencies
                .into_iter()
                .filter_map(|dep| {
                    let kind = match dep.relation_type {
                        REQUIRED_DEPENDENCY => DependencyKind::Required,
                        OPTIONAL_DEPENDENCY => DependencyKind::Optional,
                        EMBEDDED_DEPENDENCY => DependencyKind::Embedded,
                        INCOMPATIBLE_DEPENDENCY => DependencyKind::Incompatible,
                        _ => return None,
                    };
                    Some(Dependency {
                        project_id: Some(dep.mod_id.to_string()),
                        version_id: None,
                        kind,
                    })
                })
                .collect(),
            loaders: file
                .game_versions
                .iter()
                .map(|name| name.to_lowercase())
                .filter(|name| matches!(name.as_str(), "forge" | "neoforge" | "fabric" | "quilt"))
                .collect(),
            game_versions: file
                .game_versions
                .into_iter()
                .filter(|name| name.starts_with(|c: char| c.is_ascii_digit()))
                .collect(),
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FingerprintMatches {
    #[serde(default)]
    exact_matches: Vec<FingerprintMatch>,
}

#[derive(Deserialize)]
struct FingerprintMatch {
    id: u64,
    file: MatchedFile,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MatchedFile {
    id: u64,
    file_fingerprint: u32,
}

/// CurseForge's file fingerprint: MurmurHash2 with seed 1 over the bytes without whitespace.
pub fn fingerprint(bytes: &[u8]) -> u32 {
    const M: u32 = 0x5bd1_e995;
    let data: Vec<u8> = bytes
        .iter()
        .copied()
        .filter(|byte| !matches!(byte, 9 | 10 | 13 | 32))
        .collect();
    let mut hash = 1 ^ data.len() as u32;
    let mut chunks = data.chunks_exact(4);
    for chunk in &mut chunks {
        let mut k = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        k = k.wrapping_mul(M);
        k ^= k >> 24;
        k = k.wrapping_mul(M);
        hash = hash.wrapping_mul(M) ^ k;
    }
    let rest = chunks.remainder();
    if !rest.is_empty() {
        for (index, byte) in rest.iter().enumerate() {
            hash ^= u32::from(*byte) << (8 * index);
        }
        hash = hash.wrapping_mul(M);
    }
    hash ^= hash >> 13;
    hash = hash.wrapping_mul(M);
    hash ^ (hash >> 15)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_files_and_keeps_blocked_downloads_empty()
    -> std::result::Result<(), serde_json::Error> {
        let file: ApiFile = serde_json::from_str(
            r#"{"id":9254760,"modId":238222,"displayName":"19.69.0.456 for NeoForge 1.21.1",
                "fileName":"jei-1.21.1-neoforge-19.69.0.456.jar","releaseType":2,"fileDate":"2026-10-19T11:36:07Z",
                "downloadUrl":null,"fileLength":7350892,
                "hashes":[{"value":"a5d3","algo":1},{"value":"8718","algo":2}],
                "dependencies":[{"modId":1,"relationType":3},{"modId":2,"relationType":2},{"modId":3,"relationType":5}]}"#,
        )?;
        let version = ProjectVersion::try_from(file).map_err(serde::de::Error::custom)?;
        assert_eq!(version.release_type, ReleaseType::Beta);
        assert_eq!(version.files[0].sha1.as_deref(), Some("a5d3"));
        assert!(version.files[0].url.is_empty());
        assert_eq!(version.dependencies.len(), 3);
        assert!(version.dependencies[0].is_required());
        assert_eq!(version.dependencies[2].kind, DependencyKind::Incompatible);
        Ok(())
    }

    #[tokio::test]
    async fn fails_clearly_without_an_api_key() -> Result<()> {
        let curseforge = CurseForge::new(HttpClient::new(&Default::default())?, None);
        let error = curseforge
            .version("1", "2")
            .await
            .err()
            .map(|err| err.to_string());
        assert!(error.is_some_and(|message| message.contains("API key")));
        Ok(())
    }
}
