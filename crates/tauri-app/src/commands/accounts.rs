use std::sync::atomic::Ordering;

use jelauncher_core::auth::{self, Account, AccountSummary, DeviceCodePrompt, MicrosoftAuth};
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

#[tauri::command]
#[specta::specta]
pub fn add_offline_account(
    state: State<'_, AppState>,
    username: String,
) -> CommandResult<AccountSummary> {
    store_account(&state, auth::offline_account(&username)?)
}

/// Starts a Microsoft sign-in and returns the code the user enters on the Microsoft page.
#[tauri::command]
#[specta::specta]
pub async fn begin_microsoft_login(state: State<'_, AppState>) -> CommandResult<DeviceCodePrompt> {
    let code = microsoft_auth(&state)?.request_device_code().await?;
    let attempt = state.login_attempt.fetch_add(1, Ordering::SeqCst) + 1;
    let prompt = code.prompt.clone();
    *state
        .pending_login
        .lock()
        .map_err(|_| CommandError::poisoned())? = Some((attempt, code));
    Ok(prompt)
}

/// Waits until the user approves the code from [`begin_microsoft_login`], then adds the account.
#[tauri::command]
#[specta::specta]
pub async fn finish_microsoft_login(state: State<'_, AppState>) -> CommandResult<AccountSummary> {
    let (attempt, code) = state
        .pending_login
        .lock()
        .map_err(|_| CommandError::poisoned())?
        .take()
        .ok_or_else(|| CommandError::new("no Microsoft sign-in is in progress"))?;
    let account = microsoft_auth(&state)?
        .wait_for_login(&code, || {
            state.login_attempt.load(Ordering::SeqCst) != attempt
        })
        .await?;
    store_account(&state, account)
}

#[tauri::command]
#[specta::specta]
pub fn cancel_microsoft_login(state: State<'_, AppState>) {
    state.login_attempt.fetch_add(1, Ordering::SeqCst);
}

/// Returns the default account, signing a Microsoft account in again if its token expired.
pub async fn launch_account(state: &AppState) -> CommandResult<Account> {
    let account = state
        .accounts
        .read()
        .map_err(|_| CommandError::poisoned())?
        .default_account()
        .cloned()
        .ok_or_else(|| CommandError::new("add an account before launching"))?;
    if !auth::needs_refresh(&account) {
        return Ok(account);
    }
    let refreshed = microsoft_auth(state)?.refresh(&account).await?;
    store_account(state, refreshed.clone())?;
    Ok(refreshed)
}

fn microsoft_auth(state: &AppState) -> CommandResult<MicrosoftAuth<'_>> {
    let client_id = state.microsoft_client_id.ok_or_else(|| {
        CommandError::new(
            "Microsoft sign-in is not available in this build: no client id is configured",
        )
    })?;
    Ok(MicrosoftAuth {
        http: &state.http,
        client_id,
    })
}

fn store_account(state: &AppState, account: Account) -> CommandResult<AccountSummary> {
    let id = account.id.clone();
    let mut accounts = state
        .accounts
        .write()
        .map_err(|_| CommandError::poisoned())?;
    accounts.upsert(account);
    accounts.save(&state.paths.accounts_file())?;
    accounts
        .summaries()
        .into_iter()
        .find(|summary| summary.id == id)
        .ok_or_else(|| jelauncher_core::Error::UnknownAccount(id).into())
}
