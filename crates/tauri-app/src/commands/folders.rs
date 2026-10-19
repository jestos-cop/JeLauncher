use serde::Deserialize;
use tauri::{AppHandle, State};

use crate::error::CommandResult;
use crate::state::AppState;

#[derive(Debug, Clone, Copy, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum LauncherFolder {
    Root,
    Instances,
    Java,
}

#[tauri::command]
#[specta::specta]
pub fn open_launcher_folder(
    app: AppHandle,
    state: State<'_, AppState>,
    folder: LauncherFolder,
) -> CommandResult<()> {
    let path = match folder {
        LauncherFolder::Root => state.paths.root().to_path_buf(),
        LauncherFolder::Instances => state.paths.instances(),
        LauncherFolder::Java => state.paths.java(),
    };
    super::open_folder(&app, &path)
}
