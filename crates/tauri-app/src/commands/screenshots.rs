use jelauncher_core::instance::Instance;
use jelauncher_core::screenshots::{self, Screenshot};
use tauri::{AppHandle, State};

use crate::error::CommandResult;
use crate::state::AppState;

fn screenshots_dir(state: &AppState, instance_id: &str) -> CommandResult<std::path::PathBuf> {
    Ok(Instance::open(&state.paths.instances(), instance_id)?.screenshots_dir())
}

#[tauri::command]
#[specta::specta]
pub fn list_screenshots(
    state: State<'_, AppState>,
    instance_id: String,
) -> CommandResult<Vec<Screenshot>> {
    Ok(screenshots::list_screenshots(&screenshots_dir(
        &state,
        &instance_id,
    )?)?)
}

#[tauri::command]
#[specta::specta]
pub fn delete_screenshot(
    state: State<'_, AppState>,
    instance_id: String,
    file_name: String,
) -> CommandResult<()> {
    Ok(screenshots::delete_screenshot(
        &screenshots_dir(&state, &instance_id)?,
        &file_name,
    )?)
}

#[tauri::command]
#[specta::specta]
pub fn rename_screenshot(
    state: State<'_, AppState>,
    instance_id: String,
    file_name: String,
    new_name: String,
) -> CommandResult<String> {
    Ok(screenshots::rename_screenshot(
        &screenshots_dir(&state, &instance_id)?,
        &file_name,
        &new_name,
    )?)
}

#[tauri::command]
#[specta::specta]
pub fn open_screenshots_folder(
    app: AppHandle,
    state: State<'_, AppState>,
    instance_id: String,
) -> CommandResult<()> {
    super::open_folder(&app, &screenshots_dir(&state, &instance_id)?)
}
