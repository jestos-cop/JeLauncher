use jelauncher_core::instance::Instance;
use jelauncher_core::mods::{self, ModFile};
use tauri::State;

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;

fn mods_dir(state: &AppState, instance_id: &str) -> CommandResult<std::path::PathBuf> {
    Ok(Instance::open(&state.paths.instances(), instance_id)?.mods_dir())
}

#[tauri::command]
#[specta::specta]
pub async fn list_mods(
    state: State<'_, AppState>,
    instance_id: String,
) -> CommandResult<Vec<ModFile>> {
    let dir = mods_dir(&state, &instance_id)?;
    // Opening every jar is blocking file I/O.
    tauri::async_runtime::spawn_blocking(move || mods::list_mods(&dir))
        .await
        .map_err(|err| CommandError::new(err.to_string()))?
        .map_err(Into::into)
}

#[tauri::command]
#[specta::specta]
pub fn set_mod_enabled(
    state: State<'_, AppState>,
    instance_id: String,
    file_name: String,
    enabled: bool,
) -> CommandResult<String> {
    Ok(mods::set_mod_enabled(
        &mods_dir(&state, &instance_id)?,
        &file_name,
        enabled,
    )?)
}

#[tauri::command]
#[specta::specta]
pub fn remove_mod(
    state: State<'_, AppState>,
    instance_id: String,
    file_name: String,
) -> CommandResult<()> {
    Ok(mods::remove_mod(
        &mods_dir(&state, &instance_id)?,
        &file_name,
    )?)
}
