#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bindings;
mod commands;
mod error;
mod events;
mod state;

use tauri::Manager;

use crate::state::AppState;

fn main() {
    let builder = bindings::builder();
    #[cfg(debug_assertions)]
    bindings::export(&builder);

    let result = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            let state = AppState::load()?;
            // Lets the frontend display screenshots and other images stored with instances.
            let assets = app.asset_protocol_scope();
            assets.allow_directory(state.paths.instances(), true)?;
            assets.allow_directory(state.paths.icons(), true)?;
            app.manage(state);
            Ok(())
        })
        .run(tauri::generate_context!());

    if let Err(err) = result {
        eprintln!("JeLauncher failed to start: {err}");
        std::process::exit(1);
    }
}
