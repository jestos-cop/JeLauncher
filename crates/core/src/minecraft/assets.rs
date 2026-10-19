use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::net::Download;
use crate::{Error, Result};

const RESOURCES_URL: &str = "https://resources.download.minecraft.net";

#[derive(Debug, Clone, Deserialize)]
pub struct AssetIndex {
    pub objects: HashMap<String, AssetObject>,
    /// Pre-1.7 indexes expect files laid out by name under `assets/virtual/<id>`.
    #[serde(default, rename = "virtual")]
    pub is_virtual: bool,
    /// Pre-1.6 indexes expect files in `<game dir>/resources`.
    #[serde(default)]
    pub map_to_resources: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetObject {
    pub hash: String,
    pub size: u64,
}

impl AssetObject {
    fn relative_path(&self) -> String {
        format!("{}/{}", &self.hash[..2], self.hash)
    }
}

impl AssetIndex {
    pub fn downloads(&self, assets_dir: &Path) -> Vec<Download> {
        let objects = assets_dir.join("objects");
        self.objects
            .values()
            .filter(|object| object.hash.len() > 2)
            .map(|object| Download {
                url: format!("{RESOURCES_URL}/{}", object.relative_path()),
                path: objects.join(object.relative_path()),
                sha1: Some(object.hash.clone()),
                size: Some(object.size),
            })
            .collect()
    }

    /// Copies hashed objects to name-based locations for old versions that need them.
    pub fn reconstruct(&self, assets_dir: &Path, index_id: &str, game_dir: &Path) -> Result<()> {
        let target_root: PathBuf = if self.map_to_resources {
            game_dir.join("resources")
        } else if self.is_virtual {
            assets_dir.join("virtual").join(index_id)
        } else {
            return Ok(());
        };
        let objects = assets_dir.join("objects");
        for (name, object) in &self.objects {
            let target = target_root.join(name);
            if target.is_file() {
                continue;
            }
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).map_err(|err| Error::io(parent, err))?;
            }
            let source = objects.join(object.relative_path());
            std::fs::copy(&source, &target).map_err(|err| Error::io(&source, err))?;
        }
        Ok(())
    }

    /// Directory passed to the game as `${game_assets}` for legacy versions.
    pub fn legacy_dir(&self, assets_dir: &Path, index_id: &str, game_dir: &Path) -> PathBuf {
        if self.map_to_resources {
            game_dir.join("resources")
        } else {
            assets_dir.join("virtual").join(index_id)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn downloads_use_hash_layout() -> std::result::Result<(), serde_json::Error> {
        let index: AssetIndex = serde_json::from_str(
            r#"{"objects":{"icons/icon.png":{"hash":"b62ca8ec10d07e6bf5ac8dae0c8c1d2e6a1e3356","size":9101}}}"#,
        )?;
        let downloads = index.downloads(Path::new("/assets"));
        assert_eq!(
            downloads[0].url,
            "https://resources.download.minecraft.net/b6/b62ca8ec10d07e6bf5ac8dae0c8c1d2e6a1e3356"
        );
        assert_eq!(
            downloads[0].path,
            Path::new("/assets/objects/b6/b62ca8ec10d07e6bf5ac8dae0c8c1d2e6a1e3356")
        );
        assert!(!index.is_virtual);
        Ok(())
    }
}
