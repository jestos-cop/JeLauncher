#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bindings;
mod cancel;
mod cli;
mod commands;
mod error;
mod events;
mod state;

use tauri::Manager;
use tauri_specta::Event;

use crate::cli::CliRequest;
use crate::commands::folders::LogFile;
use crate::events::InstancesChangedEvent;
use crate::state::AppState;

fn main() {
    let builder = bindings::builder();
    #[cfg(debug_assertions)]
    bindings::export(&builder);

    let request = match CliRequest::parse(std::env::args().skip(1)) {
        Ok(request) => request,
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(2);
        }
    };

    let result = tauri::Builder::default()
        // Registered first: a second launch hands its arguments over and exits right away.
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
            match CliRequest::parse(args.into_iter().skip(1)) {
                Ok(Some(request)) => {
                    let _ = request.emit(app);
                }
                Ok(None) => {}
                Err(message) => log::warn!("{message}"),
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            let request = request.clone();
            builder.mount_events(app);
            let state = AppState::load()?;
            let log_file =
                jelauncher_core::launcher_log::init(&state.paths.logs(), log::LevelFilter::Info)?;
            log::info!(
                "JeLauncher {} on {} {}, data in {}",
                env!("CARGO_PKG_VERSION"),
                std::env::consts::OS,
                std::env::consts::ARCH,
                state.paths.root().display()
            );
            app.manage(LogFile(log_file));
            // Lets the frontend display screenshots and other images stored with instances.
            let assets = app.asset_protocol_scope();
            assets.allow_directory(state.paths.instances(), true)?;
            assets.allow_directory(state.paths.icons(), true)?;
            *state
                .pending_cli
                .lock()
                .map_err(|_| "state lock poisoned")? = request.clone();
            let handle = app.handle().clone();
            let watcher =
                jelauncher_core::instance::watch_instances(&state.paths.instances(), move || {
                    let _ = InstancesChangedEvent.emit(&handle);
                })?;
            app.manage(watcher);
            app.manage(state);
            Ok(())
        })
        .run(tauri::generate_context!());

    if let Err(err) = result {
        eprintln!("JeLauncher failed to start: {err}");
        std::process::exit(1);
    }
}
