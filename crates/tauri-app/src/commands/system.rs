/// Physical memory in MiB, or `None` if the platform does not report it.
#[tauri::command]
#[specta::specta]
pub async fn system_memory_mb() -> Option<u64> {
    jelauncher_core::system::total_memory_mb().await
}
