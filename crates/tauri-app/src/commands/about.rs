use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

use crate::error::{CommandError, CommandResult};

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub version: String,
    pub license: String,
    pub repository: String,
}

#[tauri::command]
#[specta::specta]
pub fn app_info() -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").into(),
        license: env!("CARGO_PKG_LICENSE").into(),
        repository: env!("CARGO_PKG_REPOSITORY").into(),
    }
}

/// Opens the source code repository in the browser.
#[tauri::command]
#[specta::specta]
pub fn open_repository(app: AppHandle) -> CommandResult<()> {
    app.opener()
        .open_url(env!("CARGO_PKG_REPOSITORY"), None::<&str>)
        .map_err(|err| CommandError::new(err.to_string()))
}
