use tauri_specta::{Builder, collect_commands, collect_events};

use crate::{commands, events};

pub fn builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            commands::accounts::add_offline_account,
            commands::accounts::list_accounts,
            commands::accounts::remove_account,
            commands::accounts::set_default_account,
            commands::instance::list_instances,
            commands::instance::create_instance,
            commands::instance::delete_instance,
            commands::instance::open_instance_folder,
            commands::instance::rename_instance,
            commands::instance::set_instance_group,
            commands::java::detect_java,
            commands::launch::kill_instance,
            commands::launch::launch_instance,
            commands::launch::list_running_instances,
            commands::meta::list_minecraft_versions,
            commands::settings::get_settings,
            commands::settings::update_settings,
        ])
        .events(collect_events![
            events::LaunchProgressEvent,
            events::InstanceLogEvent,
            events::InstanceExitedEvent,
        ])
}

#[cfg(debug_assertions)]
pub fn export(builder: &Builder<tauri::Wry>) {
    use specta_typescript::{BigIntExportBehavior, Typescript};

    let config = Typescript::default()
        .header("// @ts-nocheck")
        .bigint(BigIntExportBehavior::Number);
    if let Err(err) = builder.export(config, "../../ui/src/bindings.ts") {
        eprintln!("failed to export TypeScript bindings: {err}");
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn export_bindings() {
        super::export(&super::builder());
    }
}
