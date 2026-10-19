use jelauncher_core::instance::{self, InstanceSummary, NewInstance};
use tauri::State;

use crate::error::CommandResult;
use crate::state::AppState;

#[tauri::command]
#[specta::specta]
pub async fn list_instances(state: State<'_, AppState>) -> CommandResult<Vec<InstanceSummary>> {
    Ok(instance::discover_instances(&state.paths.instances())?)
}

#[tauri::command]
#[specta::specta]
pub async fn create_instance(
    state: State<'_, AppState>,
    new_instance: NewInstance,
) -> CommandResult<String> {
    Ok(instance::create_instance(&state.paths.instances(), new_instance)?.id)
}
