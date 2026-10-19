use tauri_specta::{Builder, collect_commands};

use crate::commands;

pub fn builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new().commands(collect_commands![
        commands::accounts::list_accounts,
        commands::accounts::remove_account,
        commands::accounts::set_default_account,
        commands::instance::list_instances,
        commands::instance::create_instance,
        commands::java::detect_java,
        commands::meta::list_minecraft_versions,
        commands::settings::get_settings,
        commands::settings::update_settings,
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
