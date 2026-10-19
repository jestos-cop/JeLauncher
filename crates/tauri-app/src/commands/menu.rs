use tauri::AppHandle;

use crate::error::{CommandError, CommandResult};
use crate::menu;

/// Rebuilds the native menu bar in the interface language.
#[tauri::command]
#[specta::specta]
pub fn set_menu_language(app: AppHandle, language: String) -> CommandResult<()> {
    if cfg!(target_os = "macos") {
        menu::build(&app, &language)
            .and_then(|menu| app.set_menu(menu))
            .map_err(|err| CommandError::new(err.to_string()))?;
    }
    Ok(())
}
