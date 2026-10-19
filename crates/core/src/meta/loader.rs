use serde::Deserialize;

use super::MetaService;
use crate::minecraft::{LoaderKind, LoaderProfile, LoaderSpec, LoaderVersion};
use crate::{Error, Result, fs};

#[derive(Deserialize)]
struct LoaderListEntry {
    loader: LoaderListVersion,
}

#[derive(Deserialize)]
struct LoaderListVersion {
    version: String,
    /// Only Fabric reports stability; Quilt marks pre-releases in the version string.
    #[serde(default)]
    stable: Option<bool>,
}

fn meta_url(kind: LoaderKind) -> Result<&'static str> {
    match kind {
        LoaderKind::Fabric => Ok("https://meta.fabricmc.net/v2"),
        LoaderKind::Quilt => Ok("https://meta.quiltmc.org/v3"),
        LoaderKind::Forge | LoaderKind::NeoForge => Err(Error::InvalidInput(format!(
            "{} profiles come from its installer",
            kind.id()
        ))),
    }
}

impl MetaService {
    /// Loader versions for a game version, newest first.
    pub async fn loader_versions(
        &self,
        kind: LoaderKind,
        game_version: &str,
    ) -> Result<Vec<LoaderVersion>> {
        if kind.uses_installer() {
            return self.installer_loader_versions(kind, game_version).await;
        }
        let url = format!("{}/versions/loader/{game_version}", meta_url(kind)?);
        let cache = self
            .cache_dir
            .join("loaders")
            .join(kind.id())
            .join(format!("versions-{game_version}.json"));
        let entries: Vec<LoaderListEntry> = self.client.get_json_cached(&url, &cache).await?;
        Ok(entries
            .into_iter()
            .map(|entry| LoaderVersion {
                stable: entry
                    .loader
                    .stable
                    .unwrap_or_else(|| !entry.loader.version.contains('-')),
                version: entry.loader.version,
            })
            .collect())
    }

    /// The loader's version profile; immutable once published, so cached forever.
    pub async fn loader_profile(
        &self,
        loader: &LoaderSpec,
        game_version: &str,
    ) -> Result<LoaderProfile> {
        let cache = self
            .cache_dir
            .join("loaders")
            .join(loader.kind.id())
            .join(format!("{game_version}-{}.json", loader.version));
        if let Some(profile) = fs::read_json(&cache)? {
            return Ok(profile);
        }
        let url = format!(
            "{}/versions/loader/{game_version}/{}/profile/json",
            meta_url(loader.kind)?,
            loader.version
        );
        let raw: serde_json::Value = self.client.get_json(&url).await?;
        fs::write_json(&cache, &raw)?;
        serde_json::from_value(raw).map_err(|err| crate::Error::json(url, err))
    }
}
