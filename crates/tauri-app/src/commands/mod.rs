pub mod accounts;
pub mod export;
pub mod instance;
pub mod java;
pub mod launch;
pub mod meta;
pub mod modplatform;
pub mod resources;
pub mod screenshots;
pub mod servers;
pub mod settings;
pub mod worlds;

use std::path::Path;

use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

use crate::error::{CommandError, CommandResult};

/// Opens a folder in the system file manager, creating it first if needed.
fn open_folder(app: &AppHandle, path: &Path) -> CommandResult<()> {
    std::fs::create_dir_all(path).map_err(|err| CommandError::new(err.to_string()))?;
    app.opener()
        .open_path(path.to_string_lossy(), None::<&str>)
        .map_err(|err| CommandError::new(err.to_string()))
}
