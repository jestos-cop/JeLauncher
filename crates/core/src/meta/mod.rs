mod forge;
mod loader;
mod manifest;
mod news;

use std::path::PathBuf;

pub use manifest::{LatestVersions, VersionEntry, VersionManifest, VersionType};
pub use news::NewsEntry;

use crate::minecraft::VersionJson;
use crate::net::{Download, HttpClient};
use crate::{Error, Result, fs};

const VERSION_MANIFEST_URL: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

/// Access to Mojang version metadata with an on-disk cache.
#[derive(Debug, Clone)]
pub struct MetaService {
    client: HttpClient,
    cache_dir: PathBuf,
}

impl MetaService {
    pub fn new(client: HttpClient, cache_dir: impl Into<PathBuf>) -> Self {
        Self {
            client,
            cache_dir: cache_dir.into(),
        }
    }

    pub async fn version_manifest(&self) -> Result<VersionManifest> {
        let cache = self.cache_dir.join("version_manifest_v2.json");
        self.client
            .get_json_cached(VERSION_MANIFEST_URL, &cache)
            .await
    }

    pub async fn version_json(&self, version_id: &str) -> Result<VersionJson> {
        let path = self
            .cache_dir
            .join("versions")
            .join(format!("{version_id}.json"));
        if let Some(cached) = fs::read_json(&path)? {
            return Ok(cached);
        }
        let manifest = self.version_manifest().await?;
        let entry = manifest
            .find(version_id)
            .ok_or_else(|| Error::UnknownVersion(version_id.to_owned()))?;
        self.client
            .download(&Download {
                url: entry.url.clone(),
                path: path.clone(),
                sha1: Some(entry.sha1.clone()),
                size: None,
            })
            .await?;
        fs::read_json(&path)?.ok_or_else(|| Error::UnknownVersion(version_id.to_owned()))
    }
}
