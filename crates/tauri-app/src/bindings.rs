use tauri_specta::{Builder, collect_commands, collect_events};

use crate::{commands, events};

pub fn builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            commands::about::app_info,
            commands::about::open_repository,
            commands::about::open_url,
            commands::accounts::add_offline_account,
            commands::accounts::begin_microsoft_login,
            commands::accounts::cancel_microsoft_login,
            commands::accounts::finish_microsoft_login,
            commands::accounts::list_accounts,
            commands::accounts::remove_account,
            commands::accounts::set_default_account,
            commands::export::export_instance,
            commands::export::export_mrpack,
            commands::export::list_export_entries,
            commands::folders::open_launcher_folder,
            commands::icons::add_custom_icon,
            commands::icons::list_custom_icons,
            commands::icons::remove_custom_icon,
            commands::icons::set_instance_icon,
            commands::instance::list_instances,
            commands::instance::copy_instance,
            commands::instance::create_instance,
            commands::instance::delete_instance,
            commands::instance::get_instance_notes,
            commands::instance::get_instance_settings,
            commands::instance::open_instance_folder,
            commands::instance::rename_instance,
            commands::instance::set_instance_group,
            commands::instance::set_instance_notes,
            commands::instance::set_instance_settings,
            commands::instance::set_instance_version,
            commands::java::detect_java,
            commands::launch::kill_instance,
            commands::launch::launch_instance,
            commands::launch::list_running_instances,
            commands::meta::list_loader_versions,
            commands::meta::list_minecraft_versions,
            commands::modplatform::install_modpack,
            commands::modplatform::import_instance,
            commands::modplatform::update_modpack,
            commands::modplatform::check_resource_updates,
            commands::modplatform::update_resource,
            commands::modplatform::install_project_version,
            commands::modplatform::list_modpack_versions,
            commands::modplatform::list_project_versions,
            commands::modplatform::search_resources,
            commands::resources::list_resources,
            commands::resources::remove_resource,
            commands::resources::export_mod_list,
            commands::screenshots::delete_screenshot,
            commands::screenshots::list_screenshots,
            commands::screenshots::open_screenshots_folder,
            commands::screenshots::rename_screenshot,
            commands::servers::add_server,
            commands::servers::list_servers,
            commands::servers::move_server_up,
            commands::servers::ping_server,
            commands::servers::remove_server,
            commands::servers::update_server,
            commands::resources::set_resource_enabled,
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::worlds::backup_world,
            commands::worlds::delete_world,
            commands::worlds::list_worlds,
            commands::worlds::open_saves_folder,
        ])
        .events(collect_events![
            events::LaunchProgressEvent,
            events::InstanceLogEvent,
            events::InstanceExitedEvent,
            events::ModpackProgressEvent,
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
