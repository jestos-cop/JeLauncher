use jelauncher_core::instance::Instance;
use jelauncher_core::worlds::{self, World};
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;

fn open_instance(state: &AppState, instance_id: &str) -> CommandResult<Instance> {
    Ok(Instance::open(&state.paths.instances(), instance_id)?)
}

#[tauri::command]
#[specta::specta]
pub fn list_worlds(state: State<'_, AppState>, instance_id: String) -> CommandResult<Vec<World>> {
    Ok(worlds::list_worlds(
        &open_instance(&state, &instance_id)?.saves_dir(),
    )?)
}

#[tauri::command]
#[specta::specta]
pub fn delete_world(
    state: State<'_, AppState>,
    instance_id: String,
    folder: String,
) -> CommandResult<()> {
    Ok(worlds::delete_world(
        &open_instance(&state, &instance_id)?.saves_dir(),
        &folder,
    )?)
}

/// Zips the world into the instance's backups folder and returns the archive path.
#[tauri::command]
#[specta::specta]
pub async fn backup_world(
    state: State<'_, AppState>,
    instance_id: String,
    folder: String,
) -> CommandResult<String> {
    let instance = open_instance(&state, &instance_id)?;
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
        .to_string();
    let backup = tauri::async_runtime::spawn_blocking(move || {
        worlds::backup_world(
            &instance.saves_dir(),
            &folder,
            &instance.backups_dir(),
            &timestamp,
        )
    })
    .await
    .map_err(|err| CommandError::new(err.to_string()))??;
    Ok(backup.to_string_lossy().into_owned())
}

#[tauri::command]
#[specta::specta]
pub fn open_saves_folder(
    app: AppHandle,
    state: State<'_, AppState>,
    instance_id: String,
) -> CommandResult<()> {
    let saves = open_instance(&state, &instance_id)?.saves_dir();
    std::fs::create_dir_all(&saves).map_err(|err| CommandError::new(err.to_string()))?;
    app.opener()
        .open_path(saves.to_string_lossy(), None::<&str>)
        .map_err(|err| CommandError::new(err.to_string()))
}
