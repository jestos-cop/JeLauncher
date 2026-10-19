use jelauncher_core::instance::{self, Instance, InstanceSummary, NewInstance};
use jelauncher_core::resources::ResourceKind;
use jelauncher_core::settings::InstanceSettings;
use tauri::State;
use tauri_plugin_opener::OpenerExt;

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
    std::fs::create_dir_all(&path).map_err(|err| CommandError::new(err.to_string()))?;
    app.opener()
        .open_path(path.to_string_lossy(), None::<&str>)
        .map_err(|err| CommandError::new(err.to_string()))
}
