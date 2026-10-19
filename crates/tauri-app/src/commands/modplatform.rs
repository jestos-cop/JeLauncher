use jelauncher_core::instance::Instance;
use jelauncher_core::modplatform::modrinth::{InstallTarget, ModpackTarget};
use jelauncher_core::modplatform::{ProjectVersion, ResourceKind, SearchPage, SearchQuery};
use tauri::{AppHandle, State};
use tauri_specta::Event;

use crate::error::{CommandError, CommandResult};
use crate::events::ModpackProgressEvent;
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

/// Installs a version plus its required dependencies; returns the installed file names.
#[tauri::command]
#[specta::specta]
pub async fn install_project_version(
    state: State<'_, AppState>,
    instance_id: String,
    kind: ResourceKind,
    version_id: String,
) -> CommandResult<Vec<String>> {
    let instance = Instance::open(&state.paths.instances(), &instance_id)?;
    let version = state.modrinth.version(&version_id).await?;
    let dir = instance.resource_dir(kind);
    let target = InstallTarget {
        dir: &dir,
        kind,
        game_version: &instance.config.minecraft_version,
        loader: instance.config.loader.as_ref().map(|loader| loader.kind),
    };
    Ok(state
        .modrinth
        .install_with_dependencies(version, &target)
        .await?)
}

/// All published versions of a modpack, newest first.
#[tauri::command]
#[specta::specta]
pub async fn list_modpack_versions(
    state: State<'_, AppState>,
    project_id: String,
) -> CommandResult<Vec<ProjectVersion>> {
    Ok(state
        .modrinth
        .versions(&project_id, ResourceKind::Modpack, None, None)
        .await?)
}

/// Installs a modpack version as a new instance and returns its id.
#[tauri::command]
#[specta::specta]
pub async fn install_modpack(
    app: AppHandle,
    state: State<'_, AppState>,
    version_id: String,
    name: Option<String>,
) -> CommandResult<String> {
    let concurrency = state
        .settings
        .read()
        .map_err(|_| CommandError::poisoned())?
        .network
        .concurrent_downloads as usize;
    let target = ModpackTarget {
        instances_dir: &state.paths.instances(),
        download_dir: &state.paths.downloads_cache(),
        name: name.filter(|name| !name.trim().is_empty()),
        concurrency,
    };
    let instance = state
        .modrinth
        .install_modpack(&version_id, &target, |progress| {
            let _ = ModpackProgressEvent { progress }.emit(&app);
        })
        .await?;
    Ok(instance.id)
}
