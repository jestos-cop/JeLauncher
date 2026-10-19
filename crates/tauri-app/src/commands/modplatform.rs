use jelauncher_core::instance::Instance;
use jelauncher_core::modplatform::{ProjectVersion, ResourceKind, SearchPage, SearchQuery};
use tauri::State;

use crate::error::CommandResult;
use crate::state::AppState;

#[tauri::command]
#[specta::specta]
pub async fn search_resources(
    state: State<'_, AppState>,
    query: SearchQuery,
) -> CommandResult<SearchPage> {
    Ok(state.modrinth.search(&query).await?)
}

/// Versions of a project that fit the instance's game version and loader.
#[tauri::command]
#[specta::specta]
pub async fn list_project_versions(
    state: State<'_, AppState>,
    instance_id: String,
    project_id: String,
    kind: ResourceKind,
) -> CommandResult<Vec<ProjectVersion>> {
    let instance = Instance::open(&state.paths.instances(), &instance_id)?;
    let loader = instance.config.loader.as_ref().map(|loader| loader.kind);
    Ok(state
        .modrinth
        .versions(
            &project_id,
            kind,
            Some(&instance.config.minecraft_version),
            loader,
        )
        .await?)
}

#[tauri::command]
#[specta::specta]
pub async fn install_project_version(
    state: State<'_, AppState>,
    instance_id: String,
    kind: ResourceKind,
    version_id: String,
) -> CommandResult<String> {
    let instance = Instance::open(&state.paths.instances(), &instance_id)?;
    let version = state.modrinth.version(&version_id).await?;
    Ok(state
        .modrinth
        .install(&version, &instance.resource_dir(kind))
        .await?)
}
