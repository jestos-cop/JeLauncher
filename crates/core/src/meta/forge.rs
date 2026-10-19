use std::collections::HashMap;

use serde::Deserialize;

use super::MetaService;
use crate::minecraft::{LoaderKind, LoaderVersion};
use crate::{Error, Result};

const FORGE_METADATA: &str =
    "https://maven.minecraftforge.net/net/minecraftforge/forge/maven-metadata.xml";
const FORGE_PROMOTIONS: &str =
    "https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json";
const NEOFORGE_VERSIONS: &str =
    "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge";

#[derive(Deserialize)]
struct Promotions {
    promos: HashMap<String, String>,
}

#[derive(Deserialize)]
struct NeoForgeVersions {
    versions: Vec<String>,
}

impl MetaService {
    pub(super) async fn installer_loader_versions(
        &self,
        kind: LoaderKind,
        game_version: &str,
    ) -> Result<Vec<LoaderVersion>> {
        match kind {
            LoaderKind::Forge => self.forge_versions(game_version).await,
            LoaderKind::NeoForge => self.neoforge_versions(game_version).await,
            LoaderKind::Fabric | LoaderKind::Quilt => Err(Error::InvalidInput(format!(
                "{} has no installer",
                kind.id()
            ))),
        }
    }

    async fn forge_versions(&self, game_version: &str) -> Result<Vec<LoaderVersion>> {
        let metadata = self.client.get_text(FORGE_METADATA).await?;
        let cache = self
            .cache_dir
            .join("loaders")
            .join("forge")
            .join("promotions.json");
        let promotions: Promotions = self
            .client
            .get_json_cached(FORGE_PROMOTIONS, &cache)
            .await?;
        let recommended = promotions
            .promos
            .get(&format!("{game_version}-recommended"));
        let mut versions = forge_versions_for(&metadata, game_version)
            .into_iter()
            .map(|version| LoaderVersion {
                stable: Some(&version) == recommended,
                version,
            })
            .collect::<Vec<_>>();
        versions.sort_by(|a, b| compare_versions(&b.version, &a.version));
        Ok(versions)
    }

    async fn neoforge_versions(&self, game_version: &str) -> Result<Vec<LoaderVersion>> {
        let cache = self
            .cache_dir
            .join("loaders")
            .join("neoforge")
            .join("versions.json");
        let all: NeoForgeVersions = self
            .client
            .get_json_cached(NEOFORGE_VERSIONS, &cache)
            .await?;
        let prefix = neoforge_prefix(game_version);
        let mut versions: Vec<_> = all
            .versions
            .into_iter()
            .filter(|version| version.starts_with(&prefix))
            .map(|version| LoaderVersion {
                stable: !version.contains("beta") && !version.contains("alpha"),
                version,
            })
            .collect();
        versions.sort_by(|a, b| compare_versions(&b.version, &a.version));
        Ok(versions)
    }
}

/// Forge publishes `<game version>-<forge version>`; returns the Forge part for one game version.
fn forge_versions_for(metadata: &str, game_version: &str) -> Vec<String> {
    let prefix = format!("{game_version}-");
    metadata
        .split("<version>")
        .skip(1)
        .filter_map(|rest| {
            rest.split_once("</version>")
                .map(|(version, _)| version.trim())
        })
        .filter_map(|version| version.strip_prefix(&prefix))
        .map(str::to_owned)
        .collect()
}

/// NeoForge drops the leading `1.` of the game version (`1.21.4` → `21.4.x`) and keeps
/// year-based versions whole (`26.3` → `26.3.0.x`).
fn neoforge_prefix(game_version: &str) -> String {
    let parts: Vec<&str> = game_version.split('.').collect();
    match parts.as_slice() {
        ["1", minor] => format!("{minor}.0."),
        ["1", minor, patch] => format!("{minor}.{patch}."),
        [year, drop] => format!("{year}.{drop}.0."),
        _ => format!("{game_version}."),
    }
}

fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    let numbers = |version: &str| -> Vec<u64> {
        version
            .split(|c: char| !c.is_ascii_digit())
            .filter_map(|part| part.parse().ok())
            .collect()
    };
    numbers(a).cmp(&numbers(b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_forge_versions_for_one_game_version() {
        let metadata = "<versions><version>1.21.4-54.1.14</version><version>1.21.41-1.0</version>\
                        <version>1.21.4-54.0.1</version><version>1.20.1-47.4.10</version></versions>";
        assert_eq!(
            forge_versions_for(metadata, "1.21.4"),
            ["54.1.14", "54.0.1"]
        );
    }

    #[test]
    fn maps_game_versions_to_neoforge_prefixes() {
        assert_eq!(neoforge_prefix("1.21.4"), "21.4.");
        assert_eq!(neoforge_prefix("1.21"), "21.0.");
        assert_eq!(neoforge_prefix("26.3"), "26.3.0.");
        assert_eq!(neoforge_prefix("26.3.1"), "26.3.1.");
    }

    #[test]
    fn orders_versions_numerically() {
        let mut versions = vec!["21.4.9", "21.4.161", "21.4.10-beta"];
        versions.sort_by(|a, b| compare_versions(b, a));
        assert_eq!(versions, ["21.4.161", "21.4.10-beta", "21.4.9"]);
    }
}
