use jelauncher_core::settings::GlobalSettings;
use tauri::State;

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;

#[tauri::command]
#[specta::specta]
pub fn get_settings(state: State<'_, AppState>) -> CommandResult<GlobalSettings> {
    let settings = state
        .settings
        .read()
        .map_err(|_| CommandError::poisoned())?;
    Ok(settings.clone())
}

#[tauri::command]
#[specta::specta]
pub fn update_settings(state: State<'_, AppState>, settings: GlobalSettings) -> CommandResult<()> {
    settings.save(&state.paths.settings_file())?;
    *state
        .settings
        .write()
        .map_err(|_| CommandError::poisoned())? = settings;
    Ok(())
}
