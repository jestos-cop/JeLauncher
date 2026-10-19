use jelauncher_core::launch::LaunchProgress;
use jelauncher_core::logs::LogLevel;
use jelauncher_core::net::DownloadProgress;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, specta::Type, tauri_specta::Event)]
#[serde(rename_all = "camelCase")]
pub struct LaunchProgressEvent {
    pub instance_id: String,
    pub progress: LaunchProgress,
}

#[derive(Debug, Clone, Serialize, specta::Type, tauri_specta::Event)]
#[serde(rename_all = "camelCase")]
pub struct InstanceLogEvent {
    pub instance_id: String,
    pub line: String,
    pub level: LogLevel,
}

#[derive(Debug, Clone, Serialize, specta::Type, tauri_specta::Event)]
#[serde(rename_all = "camelCase")]
pub struct InstanceExitedEvent {
    pub instance_id: String,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Serialize, specta::Type, tauri_specta::Event)]
pub struct ModpackProgressEvent {
    pub progress: DownloadProgress,
}
