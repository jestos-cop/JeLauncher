use jelauncher_core::auth::AccountSummary;
use tauri::State;

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;

#[tauri::command]
#[specta::specta]
pub fn list_accounts(state: State<'_, AppState>) -> CommandResult<Vec<AccountSummary>> {
    let accounts = state
        .accounts
        .read()
        .map_err(|_| CommandError::poisoned())?;
    Ok(accounts.summaries())
}

#[tauri::command]
#[specta::specta]
pub fn remove_account(state: State<'_, AppState>, id: String) -> CommandResult<()> {
    let mut accounts = state
        .accounts
        .write()
        .map_err(|_| CommandError::poisoned())?;
    accounts.remove(&id);
    Ok(accounts.save(&state.paths.accounts_file())?)
}

#[tauri::command]
#[specta::specta]
pub fn set_default_account(state: State<'_, AppState>, id: String) -> CommandResult<()> {
    let mut accounts = state
        .accounts
        .write()
        .map_err(|_| CommandError::poisoned())?;
    accounts.set_default(&id)?;
    Ok(accounts.save(&state.paths.accounts_file())?)
}
