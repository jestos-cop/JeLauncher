use std::path::Path;

use jelauncher_core::instance::{self, ExportEntry, Instance};
use jelauncher_core::modpack::{self, MrpackExport};
use tauri::State;

use crate::error::CommandResult;
use crate::state::AppState;

/// Top-level game folder entries, marked when the last export left them out.
#[tauri::command]
#[specta::specta]
pub fn list_export_entries(
    state: State<'_, AppState>,
    instance_id: String,
) -> CommandResult<Vec<ExportEntry>> {
    let instance = Instance::open(&state.paths.instances(), &instance_id)?;
    Ok(instance.export_entries()?)
}

#[tauri::command]
#[specta::specta]
pub async fn export_instance(
    state: State<'_, AppState>,
    instance_id: String,
    output: String,
    blocked_paths: Vec<String>,
) -> CommandResult<()> {
    let instance = Instance::open(&state.paths.instances(), &instance_id)?;
    Ok(instance::export_instance(
        &instance,
        Path::new(&output),
        &blocked_paths,
    )?)
}

#[tauri::command]
#[specta::specta]
pub async fn export_mrpack(
    state: State<'_, AppState>,
    instance_id: String,
    output: String,
    blocked_paths: Vec<String>,
    version: String,
    summary: Option<String>,
) -> CommandResult<()> {
    let instance = Instance::open(&state.paths.instances(), &instance_id)?;
    let options = MrpackExport {
        output: Path::new(&output),
        version,
        summary: summary.filter(|summary| !summary.trim().is_empty()),
        blocked: &blocked_paths,
    };
    Ok(modpack::export_mrpack(&state.modrinth, &instance, &options).await?)
}
