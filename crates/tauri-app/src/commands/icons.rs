use std::path::Path;

use jelauncher_core::icons::{self, CustomIcon};
use jelauncher_core::instance;
use tauri::State;

use crate::error::CommandResult;
use crate::state::AppState;

#[tauri::command]
#[specta::specta]
pub fn list_custom_icons(state: State<'_, AppState>) -> CommandResult<Vec<CustomIcon>> {
    Ok(icons::list_custom_icons(&state.paths.icons())?)
}

/// Copies an image into the icons folder and returns its icon key.
#[tauri::command]
#[specta::specta]
pub fn add_custom_icon(state: State<'_, AppState>, source: String) -> CommandResult<String> {
    Ok(icons::add_custom_icon(
        &state.paths.icons(),
        Path::new(&source),
    )?)
}

#[tauri::command]
#[specta::specta]
pub fn remove_custom_icon(state: State<'_, AppState>, key: String) -> CommandResult<()> {
    Ok(icons::remove_custom_icon(&state.paths.icons(), &key)?)
}

#[tauri::command]
#[specta::specta]
pub fn set_instance_icon(
    state: State<'_, AppState>,
    instance_id: String,
    icon_key: String,
) -> CommandResult<()> {
    Ok(instance::set_instance_icon(
        &state.paths.instances(),
        &instance_id,
        &icon_key,
    )?)
}
