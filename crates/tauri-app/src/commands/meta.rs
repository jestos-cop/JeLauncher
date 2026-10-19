use jelauncher_core::meta::{LatestVersions, NewsEntry, VersionEntry};
use jelauncher_core::minecraft::{LoaderKind, LoaderVersion};
use serde::Serialize;
use tauri::State;

use crate::error::CommandResult;
use crate::state::AppState;

#[derive(Serialize, specta::Type)]
pub struct MinecraftVersions {
    latest: LatestVersions,
    versions: Vec<VersionEntry>,
}

#[tauri::command]
#[specta::specta]
pub async fn list_minecraft_versions(
    state: State<'_, AppState>,
) -> CommandResult<MinecraftVersions> {
    let manifest = state.meta.version_manifest().await?;
    Ok(MinecraftVersions {
        latest: manifest.latest,
        versions: manifest.versions,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn list_loader_versions(
    state: State<'_, AppState>,
    kind: LoaderKind,
    minecraft_version: String,
) -> CommandResult<Vec<LoaderVersion>> {
    Ok(state.meta.loader_versions(kind, &minecraft_version).await?)
}

/// Minecraft: Java Edition news, newest first.
#[tauri::command]
#[specta::specta]
pub async fn list_news(state: State<'_, AppState>) -> CommandResult<Vec<NewsEntry>> {
    Ok(state.meta.news().await?)
}
