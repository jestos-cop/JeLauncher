use std::path::PathBuf;

use jelauncher_core::instance::Instance;
use jelauncher_core::resources::{self, ResourceFile, ResourceKind};
use tauri::State;

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;

fn resource_dir(state: &AppState, instance_id: &str, kind: ResourceKind) -> CommandResult<PathBuf> {
    Ok(Instance::open(&state.paths.instances(), instance_id)?.resource_dir(kind))
}

#[tauri::command]
#[specta::specta]
pub async fn list_resources(
    state: State<'_, AppState>,
    instance_id: String,
    kind: ResourceKind,
) -> CommandResult<Vec<ResourceFile>> {
    let dir = resource_dir(&state, &instance_id, kind)?;
    // Opening every archive is blocking file I/O.
    tauri::async_runtime::spawn_blocking(move || resources::list_resources(&dir, kind))
        .await
        .map_err(|err| CommandError::new(err.to_string()))?
        .map_err(Into::into)
}

#[tauri::command]
#[specta::specta]
pub fn set_resource_enabled(
    state: State<'_, AppState>,
    instance_id: String,
    kind: ResourceKind,
    file_name: String,
    enabled: bool,
) -> CommandResult<String> {
    let dir = resource_dir(&state, &instance_id, kind)?;
    Ok(resources::set_resource_enabled(&dir, &file_name, enabled)?)
}

#[tauri::command]
#[specta::specta]
pub fn remove_resource(
    state: State<'_, AppState>,
    instance_id: String,
    kind: ResourceKind,
    file_name: String,
) -> CommandResult<()> {
    Ok(resources::remove_resource(
        &resource_dir(&state, &instance_id, kind)?,
        &file_name,
    )?)
}
