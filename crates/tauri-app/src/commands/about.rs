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

/// Opens a web page in the browser; only http(s) links are allowed.
#[tauri::command]
#[specta::specta]
pub fn open_url(app: AppHandle, url: String) -> CommandResult<()> {
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err(CommandError::new(format!("not a web link: {url}")));
    }
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|err| CommandError::new(err.to_string()))
}
