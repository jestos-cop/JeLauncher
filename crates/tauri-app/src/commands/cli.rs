use tauri::State;

use crate::cli::CliRequest;
use crate::error::{CommandError, CommandResult};
use crate::state::AppState;

/// The command line request the launcher started with, handed out once.
#[tauri::command]
#[specta::specta]
pub fn take_cli_request(state: State<'_, AppState>) -> CommandResult<Option<CliRequest>> {
    Ok(state
        .pending_cli
        .lock()
        .map_err(|_| CommandError::poisoned())?
        .take())
}
