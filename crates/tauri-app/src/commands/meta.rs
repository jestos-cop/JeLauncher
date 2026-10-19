use jelauncher_core::meta::{LatestVersions, VersionEntry};
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
