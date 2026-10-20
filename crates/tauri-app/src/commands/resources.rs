use std::path::PathBuf;

use jelauncher_core::instance::Instance;
use jelauncher_core::resources::{self, ModListFormat, ResourceFile, ResourceKind};
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
    let instance = Instance::open(&state.paths.instances(), &instance_id)?;
    let tracks = super::modplatform::tracks_sources(&state)?;
    // Opening every archive is blocking file I/O.
    tauri::async_runtime::spawn_blocking(move || annotated_resources(&instance, kind, tracks))
        .await
        .map_err(|err| CommandError::new(err.to_string()))?
}

/// Files of `kind` in a resource folder with their sources, links and compatibility.
fn annotated_resources(
    instance: &Instance,
    kind: ResourceKind,
    tracks_sources: bool,
) -> CommandResult<Vec<ResourceFile>> {
    let dir = instance.resource_dir(kind);
    let mut files = resources::list_resources(&dir, kind)?;
    if tracks_sources {
        resources::attach_sources(&dir, kind, &mut files)?;
        resources::attach_compatibility(&mut files, &instance.config.minecraft_version);
    }
    if kind == ResourceKind::Mod {
        resources::attach_relations(&mut files);
    }
    Ok(files)
}

/// Files that should be switched along when `file_names` are enabled or disabled, because they
/// require them or are required by them.
#[tauri::command]
#[specta::specta]
pub async fn resources_affected_by_toggle(
    state: State<'_, AppState>,
    instance_id: String,
    kind: ResourceKind,
    file_names: Vec<String>,
    enable: bool,
) -> CommandResult<Vec<String>> {
    let instance = Instance::open(&state.paths.instances(), &instance_id)?;
    let tracks = super::modplatform::tracks_sources(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        let files = annotated_resources(&instance, kind, tracks)?;
        Ok(resources::affected_by_toggle(&files, &file_names, enable))
    })
    .await
    .map_err(|err| CommandError::new(err.to_string()))?
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
    let dir = resource_dir(&state, &instance_id, kind)?;
    resources::remove_resource(&dir, &file_name)?;
    Ok(resources::remove_source(&dir, kind, &file_name)?)
}

/// Writes the instance's enabled mods as a list in `format` to `output`.
#[tauri::command]
#[specta::specta]
pub async fn export_mod_list(
    state: State<'_, AppState>,
    instance_id: String,
    format: ModListFormat,
    output: String,
) -> CommandResult<()> {
    let dir = resource_dir(&state, &instance_id, ResourceKind::Mod)?;
    let files = tauri::async_runtime::spawn_blocking(move || {
        resources::list_resources(&dir, ResourceKind::Mod)
    })
    .await
    .map_err(|err| CommandError::new(err.to_string()))??;
    let list = resources::format_mod_list(&files, format);
    std::fs::write(&output, list).map_err(|err| CommandError::new(err.to_string()))
}

/// Turns update checks off or on for one resource with a known source.
#[tauri::command]
#[specta::specta]
pub fn set_resource_update_lock(
    state: State<'_, AppState>,
    instance_id: String,
    kind: ResourceKind,
    file_name: String,
    locked: bool,
) -> CommandResult<()> {
    let dir = resource_dir(&state, &instance_id, kind)?;
    Ok(resources::set_update_lock(&dir, kind, &file_name, locked)?)
}

/// Forgets where a resource came from, keeping the file.
#[tauri::command]
#[specta::specta]
pub fn forget_resource_source(
    state: State<'_, AppState>,
    instance_id: String,
    kind: ResourceKind,
    file_name: String,
) -> CommandResult<()> {
    let dir = resource_dir(&state, &instance_id, kind)?;
    Ok(resources::remove_source(&dir, kind, &file_name)?)
}
