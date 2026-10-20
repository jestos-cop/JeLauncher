use serde::{Deserialize, Serialize};

use super::{ProjectHit, ProjectVersion, Provider, ReleaseType, SearchPage, SearchQuery};
use crate::minecraft::{LoaderKind, LoaderSpec};
use crate::net::HttpClient;
use crate::{Error, Result};

pub const DOWNLOAD_SERVER: &str = "https://download.nodecdn.net/containers/atl";
const API: &str = "https://api.atlauncher.com/v1";

/// Client for ATLauncher packs: a static pack index plus one manifest per pack version.
#[derive(Debug, Clone)]
pub struct AtLauncher {
    http: HttpClient,
}

/// An optional mod of a pack version the user can pick before installing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct OptionalMod {
    pub name: String,
    pub description: String,
    /// Picked unless the user unticks it.
    pub selected: bool,
    pub recommended: bool,
}

/// A pack version manifest (`Configs.json`).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlPackVersion {
    pub version: String,
    pub minecraft: String,
    #[serde(default)]
    pub no_configs: bool,
    pub loader: Option<AtlLoader>,
    #[serde(default)]
    pub mods: Vec<AtlMod>,
    pub configs: Option<AtlConfigs>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AtlLoader {
    #[serde(rename = "type")]
    pub kind: String,
    pub metadata: Option<AtlLoaderMetadata>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AtlLoaderMetadata {
    pub version: Option<String>,
    pub loader: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlMod {
    pub name: String,
    #[serde(default)]
    pub version: String,
    pub url: String,
    pub file: String,
    #[serde(default)]
    pub md5: String,
    pub filesize: Option<u64>,
    pub download: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub extract_to: Option<String>,
    pub extract_folder: Option<String>,
    pub decomp_type: Option<String>,
    pub decomp_file: Option<String>,
    pub description: Option<String>,
    #[serde(default = "yes")]
    pub client: bool,
    #[serde(default)]
    pub optional: bool,
    #[serde(default)]
    pub selected: bool,
    #[serde(default)]
    pub recommended: bool,
    #[serde(default)]
    pub hidden: bool,
    pub group: Option<String>,
    #[serde(default)]
    pub depends: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AtlConfigs {
    pub sha1: Option<String>,
}

fn yes() -> bool {
    true
}

impl AtlPackVersion {
    pub fn loader_spec(&self) -> Result<Option<LoaderSpec>> {
        let Some(loader) = &self.loader else {
            return Ok(None);
        };
        let kind = match loader.kind.as_str() {
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
        // Fabric and Quilt name the loader version; Forge-style loaders the version itself.
        let version = loader.metadata.as_ref().and_then(|metadata| match kind {
            LoaderKind::Fabric | LoaderKind::Quilt => metadata.loader.clone(),
            LoaderKind::Forge | LoaderKind::NeoForge => metadata.version.clone(),
        });
        let version = version
            .filter(|version| !version.is_empty())
            .ok_or_else(|| {
                Error::InvalidInput(format!("the pack does not pin a {kind:?} version"))
            })?;
        Ok(Some(LoaderSpec { kind, version }))
    }

    /// Optional client mods the user can choose from; hidden ones follow their dependants.
    pub fn optional_mods(&self) -> Vec<OptionalMod> {
        self.mods
            .iter()
            .filter(|module| module.client && module.optional && !module.hidden)
            .map(|module| OptionalMod {
                name: module.name.clone(),
                description: module.description.clone().unwrap_or_default(),
                selected: module.selected,
                recommended: module.recommended,
            })
            .collect()
    }

    /// Client mods to install for the chosen optional mods (`None` keeps the pack defaults),
    /// adding what they depend on and keeping one mod per exclusive group.
    pub fn mods_to_install(&self, chosen: Option<&[String]>) -> Vec<&AtlMod> {
        let mut picked: Vec<&str> = self
            .mods
            .iter()
            .filter(|module| module.client && module.optional)
            .filter(|module| match chosen {
                Some(chosen) => chosen.contains(&module.name),
                None => module.selected,
            })
            .map(|module| module.name.as_str())
            .collect();
        let mut index = 0;
        while let Some(name) = picked.get(index).copied() {
            if let Some(module) = self.mods.iter().find(|module| module.name == name) {
                for dependency in &module.depends {
                    if !picked.contains(&dependency.as_str()) {
                        picked.push(dependency);
                    }
                }
            }
            index += 1;
        }
        let mut groups_taken = Vec::new();
        self.mods
            .iter()
            .filter(|module| module.client)
            .filter(|module| !module.optional || picked.contains(&module.name.as_str()))
            .filter(|module| match &module.group {
                Some(group) if module.optional => {
                    let first = !groups_taken.contains(group);
                    groups_taken.push(group.clone());
                    first
                }
                _ => true,
            })
            .collect()
    }
}

impl AtLauncher {
    pub fn new(http: HttpClient) -> Self {
        Self { http }
    }

    /// Public packs whose name contains the query, in the index's order.
    pub async fn search(&self, query: &SearchQuery) -> Result<SearchPage> {
        let text = query.text.trim().to_lowercase();
        let packs: Vec<IndexedPack> = self
            .index()
            .await?
            .into_iter()
            .filter(|pack| pack.kind == "public" && !pack.system && !pack.versions.is_empty())
            .filter(|pack| pack.name.to_lowercase().contains(&text))
            .collect();
        Ok(SearchPage {
            total: packs.len() as u32,
            hits: packs
                .into_iter()
                .skip(query.offset as usize)
                .take(query.limit as usize)
                .map(Into::into)
                .collect(),
        })
    }

    /// Published versions of a pack, newest first.
    pub async fn versions(&self, pack_id: &str) -> Result<Vec<ProjectVersion>> {
        Ok(self
            .indexed_pack(pack_id)
            .await?
            .versions
            .into_iter()
            .map(|version| ProjectVersion {
                id: version.version.clone(),
                project_id: pack_id.to_owned(),
                name: format!("Minecraft {}", version.minecraft),
                version_number: version.version,
                release_type: ReleaseType::Release,
                published: String::new(),
                changelog: None,
                files: Vec::new(),
                dependencies: Vec::new(),
                loaders: Vec::new(),
                game_versions: Vec::new(),
            })
            .collect())
    }

    pub async fn version(&self, pack_id: &str, version: &str) -> Result<ProjectVersion> {
        self.versions(pack_id)
            .await?
            .into_iter()
            .find(|candidate| candidate.id == version)
            .ok_or_else(|| {
                Error::InvalidInput(format!(
                    "ATLauncher pack {pack_id} has no version {version}"
                ))
            })
    }

    pub async fn pack_version(&self, pack_id: &str, version: &str) -> Result<AtlPackVersion> {
        self.http
            .get_json(&format!(
                "{DOWNLOAD_SERVER}/packs/{pack_id}/versions/{version}/Configs.json"
            ))
            .await
    }

    /// The optional mods a share code turns on, after checking it belongs to this pack version.
    pub async fn share_code(
        &self,
        pack_id: &str,
        version: &str,
        code: &str,
    ) -> Result<Vec<String>> {
        let response: ShareCodeResponse = self
            .http
            .get_json(&format!("{API}/share-codes/{}", code.trim()))
            .await?;
        let data = match (response.error, response.data) {
            (false, Some(data)) => data,
            _ => {
                return Err(Error::InvalidInput(
                    response
                        .message
                        .unwrap_or_else(|| "unknown share code".into()),
                ));
            }
        };
        if safe_name(&data.pack) != pack_id || data.version != version {
            return Err(Error::InvalidInput(format!(
                "the share code is for {} {}",
                data.pack, data.version
            )));
        }
        Ok(data
            .mods
            .optional
            .into_iter()
            .filter(|module| module.selected)
            .map(|module| module.name)
            .collect())
    }

    /// The display name of a pack, e.g. `Player's Assembly` for `PlayersAssembly`.
    pub async fn pack_name(&self, pack_id: &str) -> Result<String> {
        Ok(self.indexed_pack(pack_id).await?.name)
    }

    async fn indexed_pack(&self, pack_id: &str) -> Result<IndexedPack> {
        self.index()
            .await?
            .into_iter()
            .find(|pack| safe_name(&pack.name) == pack_id)
            .ok_or_else(|| Error::InvalidInput(format!("unknown ATLauncher pack {pack_id}")))
    }

    async fn index(&self) -> Result<Vec<IndexedPack>> {
        self.http
            .get_json(&format!("{DOWNLOAD_SERVER}/launcher/json/packsnew.json"))
            .await
    }
}

/// The folder name ATLauncher uses for a pack: its name without anything but letters and digits.
fn safe_name(name: &str) -> String {
    name.chars().filter(char::is_ascii_alphanumeric).collect()
}

#[derive(Deserialize)]
struct IndexedPack {
    name: String,
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    system: bool,
    description: Option<String>,
    #[serde(default)]
    versions: Vec<IndexedVersion>,
}

#[derive(Deserialize)]
struct IndexedVersion {
    version: String,
    minecraft: String,
}

impl From<IndexedPack> for ProjectHit {
    fn from(pack: IndexedPack) -> Self {
        let id = safe_name(&pack.name);
        Self {
            provider: Provider::AtLauncher,
            icon_url: Some(format!(
                "{DOWNLOAD_SERVER}/launcher/images/{}.png",
                id.to_lowercase()
            )),
            slug: id.clone(),
            id,
            title: pack.name,
            description: pack.description.unwrap_or_default(),
            author: String::new(),
            downloads: 0,
        }
    }
}

#[derive(Deserialize)]
struct ShareCodeResponse {
    error: bool,
    message: Option<String>,
    data: Option<ShareCode>,
}

#[derive(Deserialize)]
struct ShareCode {
    pack: String,
    version: String,
    mods: ShareCodeMods,
}

#[derive(Deserialize)]
struct ShareCodeMods {
    #[serde(default)]
    optional: Vec<ShareCodeMod>,
}

#[derive(Deserialize)]
struct ShareCodeMod {
    name: String,
    selected: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pack(mods: &str) -> AtlPackVersion {
        serde_json::from_str(&format!(
            r#"{{"version":"1","minecraft":"1.21.1","mods":{mods}}}"#
        ))
        .unwrap_or_else(|err| panic!("{err}"))
    }

    fn module(name: &str, extra: &str) -> String {
        format!(
            r#"{{"name":"{name}","url":"u","file":"{name}.jar","download":"server","type":"mods"{extra}}}"#
        )
    }

    fn names(mods: Vec<&AtlMod>) -> Vec<&str> {
        mods.into_iter()
            .map(|module| module.name.as_str())
            .collect()
    }

    #[test]
    fn picks_dependencies_and_one_mod_per_group() {
        let pack = pack(&format!(
            "[{},{},{},{},{},{}]",
            module("core", ""),
            module(
                "minimap",
                r#","optional":true,"selected":true,"depends":["lib"]"#
            ),
            module("lib", r#","optional":true,"hidden":true"#),
            module("shaders-a", r#","optional":true,"group":"shaders""#),
            module("shaders-b", r#","optional":true,"group":"shaders""#),
            module("server-only", r#","client":false"#),
        ));
        assert_eq!(
            names(pack.mods_to_install(None)),
            ["core", "minimap", "lib"]
        );
        let chosen = ["shaders-a".to_owned(), "shaders-b".to_owned()];
        assert_eq!(
            names(pack.mods_to_install(Some(&chosen))),
            ["core", "shaders-a"]
        );
        let offered: Vec<_> = pack
            .optional_mods()
            .into_iter()
            .map(|module| module.name)
            .collect();
        assert_eq!(offered, ["minimap", "shaders-a", "shaders-b"]);
    }

    #[test]
    fn safe_names_drop_punctuation() {
        assert_eq!(safe_name("Player's Assembly"), "PlayersAssembly");
    }
}
