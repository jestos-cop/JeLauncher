use serde::Deserialize;
use serde::de::DeserializeOwned;

use super::{ProjectHit, ProjectVersion, Provider, ReleaseType, SearchPage, SearchQuery};
use crate::net::HttpClient;
use crate::{Error, Result};

const API: &str = "https://api.technicpack.net";
/// The platform asks launchers to identify themselves with a build name.
const BUILD: &str = "jelauncher";

/// Client for Technic modpacks. Packs are either one zip or a Solder build of many mod zips.
#[derive(Debug, Clone)]
pub struct Technic {
    http: HttpClient,
    /// Platform client id; only needed for private packs.
    client_id: Option<String>,
}

/// Where the files of one Technic pack version come from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TechnicFiles {
    /// One archive with the whole game directory.
    Zip(String),
    /// Mod archives that are extracted on top of each other.
    Solder(Vec<SolderMod>),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SolderMod {
    pub name: String,
    pub url: String,
    pub md5: String,
    #[serde(default, rename = "filesize")]
    pub size: Option<u64>,
}

/// Everything needed to create an instance for one Technic pack version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TechnicPackVersion {
    pub pack_name: String,
    pub version: String,
    pub minecraft_version: String,
    pub files: TechnicFiles,
}

impl Technic {
    pub fn new(http: HttpClient) -> Self {
        Self::with_client_id(http, None)
    }

    pub fn with_client_id(http: HttpClient, client_id: Option<String>) -> Self {
        Self {
            http,
            client_id: client_id.filter(|id| !id.trim().is_empty()),
        }
    }

    /// GET on the platform API with the build name and, when set, the client id.
    async fn get<T: DeserializeOwned>(&self, path: &str, extra: &[(&str, String)]) -> Result<T> {
        let mut query = vec![("build", BUILD.to_owned())];
        if let Some(client_id) = &self.client_id {
            query.push(("cid", client_id.clone()));
        }
        query.extend_from_slice(extra);
        self.http
            .get_json_query(&format!("{API}/{path}"), &query)
            .await
    }

    /// Searches by name, or lists trending packs for an empty query. A pack slug or
    /// `api.technicpack.net/modpack/<slug>` URL finds that pack directly.
    pub async fn search(&self, query: &SearchQuery) -> Result<SearchPage> {
        let text = query.text.trim();
        if let Some(slug) = pack_slug_from_url(text) {
            let pack = self.pack(slug).await?;
            return Ok(SearchPage {
                total: 1,
                hits: vec![pack.into()],
            });
        }
        let list: ApiPackList = if text.is_empty() {
            self.get("trending", &[]).await?
        } else {
            self.get("search", &[("q", text.to_owned())]).await?
        };
        Ok(SearchPage {
            total: list.modpacks.len() as u32,
            hits: list
                .modpacks
                .into_iter()
                .skip(query.offset as usize)
                .take(query.limit as usize)
                .map(Into::into)
                .collect(),
        })
    }

    /// Published versions of a pack, newest first; packs without Solder have just one.
    pub async fn versions(&self, slug: &str) -> Result<Vec<ProjectVersion>> {
        let pack = self.pack(slug).await?;
        let Some(solder) = &pack.solder else {
            let version = pack.version.clone().unwrap_or_else(|| "latest".into());
            return Ok(vec![project_version(slug, &version, &version)]);
        };
        let builds: SolderPack = self
            .http
            .get_json(&format!("{}/modpack/{slug}", solder.trim_end_matches('/')))
            .await?;
        Ok(builds
            .builds
            .iter()
            .rev()
            .map(|build| {
                let name = if Some(build) == builds.recommended.as_ref() {
                    "Recommended"
                } else {
                    build
                };
                project_version(slug, build, name)
            })
            .collect())
    }

    pub async fn version(&self, slug: &str, version: &str) -> Result<ProjectVersion> {
        self.versions(slug)
            .await?
            .into_iter()
            .find(|candidate| candidate.id == version)
            .ok_or_else(|| {
                Error::InvalidInput(format!("Technic pack {slug} has no version {version}"))
            })
    }

    /// The Minecraft version and file sources of a pack version.
    pub async fn pack_version(&self, slug: &str, version: &str) -> Result<TechnicPackVersion> {
        let pack = self.pack(slug).await?;
        let (minecraft_version, files) = match (&pack.solder, &pack.url) {
            (Some(solder), _) => {
                let build: SolderBuild = self
                    .http
                    .get_json(&format!(
                        "{}/modpack/{slug}/{version}",
                        solder.trim_end_matches('/')
                    ))
                    .await?;
                (build.minecraft, TechnicFiles::Solder(build.mods))
            }
            (None, Some(url)) => (pack.minecraft.clone(), TechnicFiles::Zip(url.clone())),
            (None, None) => {
                return Err(Error::InvalidInput(format!(
                    "{} has no download",
                    pack.display_name
                )));
            }
        };
        Ok(TechnicPackVersion {
            pack_name: pack.display_name,
            version: version.to_owned(),
            minecraft_version,
            files,
        })
    }

    async fn pack(&self, slug: &str) -> Result<ApiPack> {
        self.get(&format!("modpack/{slug}"), &[]).await
    }
}

fn pack_slug_from_url(text: &str) -> Option<&str> {
    let path = text
        .strip_prefix("https://")
        .or_else(|| text.strip_prefix("http://"))?
        .strip_prefix("api.technicpack.net/modpack/")?;
    path.split(['/', '?'])
        .next()
        .filter(|slug| !slug.is_empty())
}

fn project_version(slug: &str, version: &str, name: &str) -> ProjectVersion {
    ProjectVersion {
        id: version.to_owned(),
        project_id: slug.to_owned(),
        name: name.to_owned(),
        version_number: version.to_owned(),
        release_type: ReleaseType::Release,
        published: String::new(),
        changelog: None,
        files: Vec::new(),
        dependencies: Vec::new(),
    }
}

#[derive(Deserialize)]
struct ApiPackList {
    #[serde(default)]
    modpacks: Vec<ApiListedPack>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiListedPack {
    name: String,
    slug: String,
    icon_url: Option<String>,
}

impl From<ApiListedPack> for ProjectHit {
    fn from(pack: ApiListedPack) -> Self {
        Self {
            provider: Provider::Technic,
            id: pack.slug.clone(),
            slug: pack.slug,
            title: pack.name,
            description: String::new(),
            author: String::new(),
            icon_url: pack.icon_url,
            downloads: 0,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiPack {
    name: String,
    display_name: String,
    #[serde(default)]
    user: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    minecraft: String,
    version: Option<String>,
    url: Option<String>,
    solder: Option<String>,
    #[serde(default)]
    installs: u64,
    icon: Option<ApiImage>,
}

#[derive(Deserialize)]
struct ApiImage {
    url: String,
}

impl From<ApiPack> for ProjectHit {
    fn from(pack: ApiPack) -> Self {
        Self {
            provider: Provider::Technic,
            id: pack.name.clone(),
            slug: pack.name,
            title: pack.display_name,
            description: pack.description,
            author: pack.user,
            icon_url: pack.icon.map(|icon| icon.url),
            downloads: pack.installs,
        }
    }
}

#[derive(Deserialize)]
struct SolderPack {
    #[serde(default)]
    builds: Vec<String>,
    recommended: Option<String>,
}

#[derive(Deserialize)]
struct SolderBuild {
    minecraft: String,
    #[serde(default)]
    mods: Vec<SolderMod>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_slugs_in_api_urls() {
        assert_eq!(
            pack_slug_from_url("https://api.technicpack.net/modpack/tekkit-2?build=x"),
            Some("tekkit-2")
        );
        assert_eq!(pack_slug_from_url("tekkit"), None);
        assert_eq!(
            pack_slug_from_url("https://www.technicpack.net/modpack/tekkit"),
            None
        );
    }
}
