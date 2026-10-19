use jelauncher_core::java::{self, JavaInstall};
use tauri::State;

use crate::error::CommandResult;
use crate::state::AppState;

#[tauri::command]
#[specta::specta]
pub async fn detect_java(state: State<'_, AppState>) -> CommandResult<Vec<JavaInstall>> {
    Ok(java::detect_java_installs(&state.paths.java()).await)
}
