use tauri::State;

use crate::error::CommandResult;
use crate::state::AppState;

/// Physical memory in MiB, or `None` if the platform does not report it.
#[tauri::command]
#[specta::specta]
pub async fn system_memory_mb() -> Option<u64> {
    jelauncher_core::system::total_memory_mb().await
}

/// Publishes a log on mclo.gs and returns its URL.
#[tauri::command]
#[specta::specta]
pub async fn upload_log(state: State<'_, AppState>, content: String) -> CommandResult<String> {
    Ok(jelauncher_core::paste::upload_log(&state.http, &content).await?)
}
