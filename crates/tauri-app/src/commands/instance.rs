use jelauncher_core::instance::{self, CopyOptions, Instance, InstanceSummary, NewInstance};
use jelauncher_core::minecraft::LoaderSpec;
use jelauncher_core::resources::ResourceKind;
use jelauncher_core::settings::InstanceSettings;
use tauri::State;

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;

#[tauri::command]
#[specta::specta]
pub async fn list_instances(state: State<'_, AppState>) -> CommandResult<Vec<InstanceSummary>> {
    Ok(instance::discover_instances(&state.paths.instances())?)
}

#[tauri::command]
#[specta::specta]
pub async fn create_instance(
    state: State<'_, AppState>,
    new_instance: NewInstance,
) -> CommandResult<String> {
    Ok(instance::create_instance(&state.paths.instances(), new_instance)?.id)
}

/// Copies an instance with the selected contents and returns the new instance's id.
#[tauri::command]
#[specta::specta]
pub async fn copy_instance(
    state: State<'_, AppState>,
    instance_id: String,
    options: CopyOptions,
) -> CommandResult<String> {
    let instances_dir = state.paths.instances();
    let source = Instance::open(&instances_dir, &instance_id)?;
    let copy = tauri::async_runtime::spawn_blocking(move || {
        instance::copy_instance(&instances_dir, &source, &options)
    })
    .await
    .map_err(|err| CommandError::new(err.to_string()))??;
    Ok(copy.id)
}

#[tauri::command]
#[specta::specta]
pub fn rename_instance(
    state: State<'_, AppState>,
    instance_id: String,
    name: String,
) -> CommandResult<()> {
    Ok(instance::rename_instance(
        &state.paths.instances(),
        &instance_id,
        &name,
    )?)
}

#[tauri::command]
#[specta::specta]
pub fn set_instance_group(
    state: State<'_, AppState>,
    instance_id: String,
    group: Option<String>,
) -> CommandResult<()> {
    Ok(instance::set_instance_group(
        &state.paths.instances(),
        &instance_id,
        group,
    )?)
}

#[tauri::command]
#[specta::specta]
pub fn get_instance_settings(
    state: State<'_, AppState>,
    instance_id: String,
) -> CommandResult<InstanceSettings> {
    Ok(Instance::open(&state.paths.instances(), &instance_id)?
        .config
        .settings)
}

#[tauri::command]
#[specta::specta]
pub fn set_instance_settings(
    state: State<'_, AppState>,
    instance_id: String,
    settings: InstanceSettings,
) -> CommandResult<()> {
    Ok(instance::set_instance_settings(
        &state.paths.instances(),
        &instance_id,
        settings,
    )?)
}

#[tauri::command]
#[specta::specta]
pub fn set_instance_version(
    state: State<'_, AppState>,
    instance_id: String,
    minecraft_version: String,
    loader: Option<LoaderSpec>,
) -> CommandResult<()> {
    Ok(instance::set_instance_version(
        &state.paths.instances(),
        &instance_id,
        &minecraft_version,
        loader,
    )?)
}

#[tauri::command]
#[specta::specta]
pub fn get_instance_notes(
    state: State<'_, AppState>,
    instance_id: String,
) -> CommandResult<String> {
    Ok(Instance::open(&state.paths.instances(), &instance_id)?
        .config
        .notes)
}

#[tauri::command]
#[specta::specta]
pub fn set_instance_notes(
    state: State<'_, AppState>,
    instance_id: String,
    notes: String,
) -> CommandResult<()> {
    Ok(instance::set_instance_notes(
        &state.paths.instances(),
        &instance_id,
        notes,
    )?)
}

#[tauri::command]
#[specta::specta]
pub fn delete_instance(state: State<'_, AppState>, instance_id: String) -> CommandResult<()> {
    if state
        .running
        .lock()
        .map_err(|_| CommandError::poisoned())?
        .contains_key(&instance_id)
    {
        return Err(CommandError::new("stop the instance before deleting it"));
    }
    Ok(instance::delete_instance(
        &state.paths.instances(),
        &instance_id,
    )?)
}

#[tauri::command]
#[specta::specta]
pub fn open_instance_folder(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    instance_id: String,
    kind: Option<ResourceKind>,
) -> CommandResult<()> {
    let instance = Instance::open(&state.paths.instances(), &instance_id)?;
    let path = kind.map_or_else(|| instance.game_dir(), |kind| instance.resource_dir(kind));
    super::open_folder(&app, &path)
}
