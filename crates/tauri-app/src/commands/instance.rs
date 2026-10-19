use jelauncher_core::instance::{self, InstanceSummary};
use jelauncher_core::paths::DataPaths;
use tauri::State;

use crate::error::CommandResult;

#[tauri::command]
#[specta::specta]
pub async fn list_instances(paths: State<'_, DataPaths>) -> CommandResult<Vec<InstanceSummary>> {
    Ok(instance::discover_instances(&paths.instances())?)
}
