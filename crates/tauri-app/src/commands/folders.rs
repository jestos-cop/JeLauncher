use jelauncher_core::instance::{INSTANCE_CONFIG_FILE, Instance};
use jelauncher_core::tools;
use serde::Deserialize;
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

use crate::error::{CommandError, CommandResult};
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

/// Path of the launcher's own log file, registered at startup.
pub struct LogFile(pub std::path::PathBuf);

/// Opens the launcher's log file in the default text viewer.
#[tauri::command]
#[specta::specta]
pub fn open_launcher_log(app: AppHandle, log_file: State<'_, LogFile>) -> CommandResult<()> {
    app.opener()
        .open_path(log_file.0.to_string_lossy(), None::<&str>)
        .map_err(|err| CommandError::new(err.to_string()))
}

#[derive(Debug, Clone, Copy, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum InstanceFile {
    Config,
    GameOptions,
}

/// Opens an instance file in the configured editor, or the system default app.
#[tauri::command]
#[specta::specta]
pub fn open_instance_file(
    app: AppHandle,
    state: State<'_, AppState>,
    instance_id: String,
    file: InstanceFile,
) -> CommandResult<()> {
    let instance = Instance::open(&state.paths.instances(), &instance_id)?;
    let path = match file {
        InstanceFile::Config => instance.root.join(INSTANCE_CONFIG_FILE),
        InstanceFile::GameOptions => instance.game_dir().join("options.txt"),
    };
    if !path.is_file() {
        return Err(CommandError::new(format!(
            "{} does not exist yet; launch the game once to create it",
            path.display()
        )));
    }
    let editor = state
        .settings
        .read()
        .map_err(|_| CommandError::poisoned())?
        .tools
        .editor
        .clone()
        .filter(|editor| !editor.trim().is_empty());
    match editor {
        Some(editor) => Ok(tools::open_in_editor(&editor, &path)?),
        None => app
            .opener()
            .open_path(path.to_string_lossy(), None::<&str>)
            .map_err(|err| CommandError::new(err.to_string())),
    }
}
