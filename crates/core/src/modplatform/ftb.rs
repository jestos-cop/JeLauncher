use futures_util::{StreamExt, TryStreamExt, stream};
use serde::Deserialize;

use super::{
    ProjectHit, ProjectVersion, Provider, ReleaseType, SearchPage, SearchQuery, SortOrder,
};
use crate::minecraft::{LoaderKind, LoaderSpec};
use crate::net::HttpClient;
use crate::{Error, Result};

const API: &str = "https://api.feed-the-beast.com/v1/modpacks/public";
const MAX_RESULTS: u32 = 50;
/// Pack list endpoints, best first.
const MOST_INSTALLED: &str = "popular/installs";
const RECENTLY_UPDATED: &str = "updated";
/// How many ranked ids are fetched to order search results; covers every public pack.
const RANKED_POOL: u32 = 500;
/// The API rate-limits bursts, so pack details are fetched a few at a time.
const PARALLEL_REQUESTS: usize = 4;

/// Client for FTB modpacks; FTB publishes whole packs only, no single mods.
#[derive(Debug, Clone)]
pub struct Ftb {
    http: HttpClient,
}

/// One file of an FTB pack version, relative to the game directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FtbFile {
    pub path: String,
    pub url: String,
    pub sha1: Option<String>,
    pub size: u64,
}

/// Everything needed to create an instance for one FTB pack version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FtbPackVersion {
    pub pack_name: String,
    pub version_name: String,
    pub minecraft_version: String,
    pub loader: Option<LoaderSpec>,
    pub files: Vec<FtbFile>,
}

impl Ftb {
    pub fn new(http: HttpClient) -> Self {
        Self { http }
    }

    /// Searches by name, or lists packs for an empty query, ordered as `query.sort` asks.
    pub async fn search(&self, query: &SearchQuery) -> Result<SearchPage> {
        let limit = (query.offset + query.limit).clamp(1, MAX_RESULTS);
        let text = query.text.trim();
        let ranking = ranking_list(query.sort);
        let ids = if text.is_empty() {
            let list = ranking.unwrap_or(MOST_INSTALLED);
            self.pack_ids(&format!("{API}/modpack/{list}/{limit}"), &[])
                .await?
        } else {
            // Ranked results need every match, not just the first page.
            let found = self
                .pack_ids(
                    &format!(
                        "{API}/modpack/search/{}",
                        if ranking.is_some() {
                            MAX_RESULTS
                        } else {
                            limit
                        }
                    ),
                    &[("term", text.to_owned())],
                )
                .await?;
            match ranking {
                Some(list) => {
                    let ranked = self
                        .pack_ids(&format!("{API}/modpack/{list}/{RANKED_POOL}"), &[])
                        .await?;
                    rank_by(found, &ranked)
                }
                None => found,
            }
        };
        let page_ids: Vec<u64> = ids
            .into_iter()
            .skip(query.offset as usize)
            .take(query.limit as usize)
            .collect();
        let packs: Vec<ApiPack> = stream::iter(page_ids.into_iter().map(|id| self.pack(id)))
            .buffered(PARALLEL_REQUESTS)
            .try_collect()
            .await?;
        Ok(SearchPage {
            total: (query.offset + packs.len() as u32).min(limit),
            hits: packs.into_iter().map(Into::into).collect(),
        })
    }

    async fn pack_ids(&self, url: &str, query: &[(&str, String)]) -> Result<Vec<u64>> {
        let ids: PackIds = self.http.get_json_query(url, query).await?;
        Ok(ids.packs)
    }

    /// Published versions of a pack, newest first.
    pub async fn versions(&self, pack_id: &str) -> Result<Vec<ProjectVersion>> {
        let pack = self.pack(parse_id(pack_id)?).await?;
        let mut versions: Vec<ApiVersion> = pack
            .versions
            .into_iter()
            .filter(|version| !version.private)
            .collect();
        versions.sort_by(|a, b| b.updated.cmp(&a.updated));
        Ok(versions
            .into_iter()
            .map(|version| version.into_project_version(pack_id))
            .collect())
    }

    pub async fn version(&self, pack_id: &str, version_id: &str) -> Result<ProjectVersion> {
        self.versions(pack_id)
            .await?
            .into_iter()
            .find(|version| version.id == version_id)
            .ok_or_else(|| {
                Error::InvalidInput(format!("FTB pack {pack_id} has no version {version_id}"))
            })
    }

    /// The game setup and client files of a pack version.
    pub async fn pack_version(&self, pack_id: &str, version_id: &str) -> Result<FtbPackVersion> {
        let pack = self.pack(parse_id(pack_id)?).await?;
        let details: ApiVersionDetails = self
            .http
            .get_json(&format!("{API}/modpack/{pack_id}/{version_id}"))
            .await?;
        let minecraft_version = details
            .targets
            .iter()
            .find(|target| target.kind == "game")
            .map(|target| target.version.clone())
            .ok_or_else(|| {
                Error::InvalidInput(format!("{} does not name a Minecraft version", pack.name))
            })?;
        let loader = details
            .targets
            .iter()
            .find(|target| target.kind == "modloader")
            .map(|target| {
                let kind = match target.name.as_str() {
                    "forge" => LoaderKind::Forge,
                    "neoforge" => LoaderKind::NeoForge,
                    "fabric" => LoaderKind::Fabric,
                    "quilt" => LoaderKind::Quilt,
                    other => {
                        return Err(Error::InvalidInput(format!(
                            "unsupported mod loader {other}"
                        )));
                    }
                };
                Ok(LoaderSpec {
                    kind,
                    version: target.version.clone(),
                })
            })
            .transpose()?;
        let files = details
            .files
            .into_iter()
            .filter(|file| !file.serveronly && !file.url.is_empty())
            .map(|file| FtbFile {
                path: format!(
                    "{}/{}",
                    file.path.trim_start_matches("./").trim_end_matches('/'),
                    file.name
                )
                .trim_start_matches('/')
                .to_owned(),
                url: file.url,
                sha1: file.sha1.filter(|sha1| !sha1.is_empty()),
                size: file.size,
            })
            .collect();
        Ok(FtbPackVersion {
            pack_name: pack.name,
            version_name: details.name,
            minecraft_version,
            loader,
            files,
        })
    }

    async fn pack(&self, id: u64) -> Result<ApiPack> {
        self.http.get_json(&format!("{API}/modpack/{id}")).await
    }
}

fn parse_id(id: &str) -> Result<u64> {
    id.parse()
        .map_err(|_| Error::InvalidInput(format!("invalid FTB pack id {id}")))
}

#[derive(Deserialize)]
struct PackIds {
    #[serde(default)]
    packs: Vec<u64>,
}

#[derive(Deserialize)]
struct ApiPack {
    id: u64,
    name: String,
    slug: Option<String>,
    #[serde(default)]
    synopsis: String,
    #[serde(default)]
    installs: u64,
    #[serde(default)]
    art: Vec<ApiArt>,
    #[serde(default)]
    authors: Vec<ApiAuthor>,
    #[serde(default)]
    versions: Vec<ApiVersion>,
}

#[derive(Deserialize)]
struct ApiArt {
    url: String,
    #[serde(rename = "type")]
    kind: String,
}

#[derive(Deserialize)]
struct ApiAuthor {
    name: String,
}

#[derive(Deserialize)]
struct ApiVersion {
    id: u64,
    name: String,
    #[serde(rename = "type", default)]
    kind: String,
    #[serde(default)]
    updated: i64,
    #[serde(default)]
    private: bool,
}

impl ApiVersion {
    fn into_project_version(self, pack_id: &str) -> ProjectVersion {
        ProjectVersion {
            id: self.id.to_string(),
            project_id: pack_id.to_owned(),
            name: self.name.clone(),
            version_number: self.name,
            release_type: match self.kind.to_ascii_lowercase().as_str() {
                "beta" => ReleaseType::Beta,
                "alpha" => ReleaseType::Alpha,
                _ => ReleaseType::Release,
            },
            published: published_date(self.updated),
            changelog: None,
            files: Vec::new(),
            dependencies: Vec::new(),
            loaders: Vec::new(),
            game_versions: Vec::new(),
        }
    }
}

#[derive(Deserialize)]
struct ApiVersionDetails {
    name: String,
    #[serde(default)]
    targets: Vec<ApiTarget>,
    #[serde(default)]
    files: Vec<ApiFile>,
}

#[derive(Deserialize)]
struct ApiTarget {
    name: String,
    version: String,
    #[serde(rename = "type")]
    kind: String,
}

#[derive(Deserialize)]
struct ApiFile {
    path: String,
    name: String,
    #[serde(default)]
    url: String,
    sha1: Option<String>,
    #[serde(default)]
    size: u64,
    #[serde(default)]
    serveronly: bool,
}

impl From<ApiPack> for ProjectHit {
    fn from(pack: ApiPack) -> Self {
        let icon_url = pack
            .art
            .iter()
            .find(|art| art.kind == "square")
            .map(|art| art.url.clone());
        Self {
            provider: Provider::Ftb,
            slug: pack.slug.unwrap_or_else(|| pack.id.to_string()),
            id: pack.id.to_string(),
            title: pack.name,
            description: pack.synopsis,
            author: pack
                .authors
                .into_iter()
                .next()
                .map(|author| author.name)
                .unwrap_or_default(),
            icon_url,
            downloads: pack.installs,
        }
    }
}

/// FTB reports Unix seconds; the UI expects a date string like the other platforms.
fn published_date(secs: i64) -> String {
    let secs = u64::try_from(secs).unwrap_or(0);
    crate::launcher_log::utc_timestamp(std::time::UNIX_EPOCH + std::time::Duration::from_secs(secs))
}

/// The pack list that orders results for `sort`; `None` keeps the API's relevance order.
fn ranking_list(sort: SortOrder) -> Option<&'static str> {
    match sort {
        SortOrder::Downloads => Some(MOST_INSTALLED),
        SortOrder::Updated => Some(RECENTLY_UPDATED),
        SortOrder::Relevance | SortOrder::Newest => None,
    }
}

/// Orders `ids` by their position in `ranking`; ids it does not list keep their order at the end.
fn rank_by(mut ids: Vec<u64>, ranking: &[u64]) -> Vec<u64> {
    ids.sort_by_key(|id| {
        ranking
            .iter()
            .position(|ranked| ranked == id)
            .unwrap_or(usize::MAX)
    });
    ids
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_versions() {
        let version = ApiVersion {
            id: 100_359,
            name: "1.11.6".into(),
            kind: "Release".into(),
            updated: 1_760_886_000,
            private: false,
        };
        let converted = version.into_project_version("100");
        assert_eq!(converted.id, "100359");
        assert_eq!(converted.release_type, ReleaseType::Release);
        assert_eq!(converted.published, "2025-10-19T15:00:00Z");
    }

    #[test]
    fn ranks_search_results_by_another_list() {
        assert_eq!(rank_by(vec![7, 3, 9, 5], &[5, 9, 1, 7]), [5, 9, 7, 3]);
    }
}
