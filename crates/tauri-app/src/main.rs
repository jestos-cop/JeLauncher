#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bindings;
mod commands;
mod error;

use jelauncher_core::paths::DataPaths;
use tauri::Manager;

fn main() {
    let builder = bindings::builder();
    #[cfg(debug_assertions)]
    bindings::export(&builder);

    let result = tauri::Builder::default()
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            app.manage(DataPaths::platform_default()?);
            Ok(())
        })
        .run(tauri::generate_context!());

    if let Err(err) = result {
        eprintln!("JeLauncher failed to start: {err}");
        std::process::exit(1);
    }
}
