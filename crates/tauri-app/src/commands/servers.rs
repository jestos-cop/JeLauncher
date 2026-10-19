use std::path::PathBuf;

use jelauncher_core::instance::Instance;
use jelauncher_core::servers::{self, Server, ServerStatus, server_index};
use tauri::State;

use crate::error::CommandResult;
use crate::state::AppState;

fn servers_file(state: &AppState, instance_id: &str) -> CommandResult<PathBuf> {
    Ok(Instance::open(&state.paths.instances(), instance_id)?.servers_file())
}

#[tauri::command]
#[specta::specta]
pub fn list_servers(state: State<'_, AppState>, instance_id: String) -> CommandResult<Vec<Server>> {
    Ok(servers::read_servers(&servers_file(&state, &instance_id)?)?)
}

#[tauri::command]
#[specta::specta]
pub fn add_server(
    state: State<'_, AppState>,
    instance_id: String,
    server: Server,
) -> CommandResult<()> {
    Ok(servers::edit_servers(
        &servers_file(&state, &instance_id)?,
        |servers| {
            servers.push(server);
            Ok(())
        },
    )?)
}

#[tauri::command]
#[specta::specta]
pub fn update_server(
    state: State<'_, AppState>,
    instance_id: String,
    index: u32,
    server: Server,
) -> CommandResult<()> {
    Ok(servers::edit_servers(
        &servers_file(&state, &instance_id)?,
        |servers| {
            let index = server_index(servers, index as usize)?;
            servers[index] = server;
            Ok(())
        },
    )?)
}

#[tauri::command]
#[specta::specta]
pub fn remove_server(
    state: State<'_, AppState>,
    instance_id: String,
    index: u32,
) -> CommandResult<()> {
    Ok(servers::edit_servers(
        &servers_file(&state, &instance_id)?,
        |servers| {
            servers.remove(server_index(servers, index as usize)?);
            Ok(())
        },
    )?)
}

/// Swaps the server at `index` with the one above it.
#[tauri::command]
#[specta::specta]
pub fn move_server_up(
    state: State<'_, AppState>,
    instance_id: String,
    index: u32,
) -> CommandResult<()> {
    Ok(servers::edit_servers(
        &servers_file(&state, &instance_id)?,
        |servers| {
            let index = server_index(servers, index as usize)?;
            if index > 0 {
                servers.swap(index - 1, index);
            }
            Ok(())
        },
    )?)
}

#[tauri::command]
#[specta::specta]
pub async fn ping_server(address: String) -> CommandResult<ServerStatus> {
    Ok(servers::ping_server(&address).await?)
}
