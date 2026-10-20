use std::collections::HashMap;

use serde::Serialize;
use tauri::menu::{Menu, MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri::{AppHandle, Runtime};
use tauri_specta::Event;

/// A menu bar command the frontend carries out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum MenuAction {
    About,
    Settings,
    NewInstance,
    ImportInstance,
    Refresh,
    OpenInstancesFolder,
    OpenLauncherFolder,
    ManageAccounts,
    News,
    SourceCode,
    Discord,
    LauncherLog,
}

impl MenuAction {
    const ALL: [Self; 12] = [
        Self::About,
        Self::Settings,
        Self::NewInstance,
        Self::ImportInstance,
        Self::Refresh,
        Self::OpenInstancesFolder,
        Self::OpenLauncherFolder,
        Self::ManageAccounts,
        Self::News,
        Self::SourceCode,
        Self::Discord,
        Self::LauncherLog,
    ];

    /// Menu item id, which is also its label key, and keyboard shortcut.
    fn spec(self) -> (&'static str, Option<&'static str>) {
        match self {
            Self::About => ("about", None),
            Self::Settings => ("settings", Some("CmdOrCtrl+,")),
            Self::NewInstance => ("new-instance", Some("CmdOrCtrl+N")),
            Self::ImportInstance => ("import-instance", Some("CmdOrCtrl+Shift+I")),
            Self::Refresh => ("refresh", Some("CmdOrCtrl+R")),
            Self::OpenInstancesFolder => ("open-instances-folder", None),
            Self::OpenLauncherFolder => ("open-launcher-folder", None),
            Self::ManageAccounts => ("manage-accounts", None),
            Self::News => ("news", None),
            Self::SourceCode => ("source-code", None),
            Self::Discord => ("discord", None),
            Self::LauncherLog => ("launcher-log", None),
        }
    }
}

#[derive(Debug, Clone, Serialize, specta::Type, tauri_specta::Event)]
pub struct MenuActionEvent {
    pub action: MenuAction,
}

/// Labels of the menu bar in one language, keyed by item id.
struct MenuText(HashMap<String, String>);

impl MenuText {
    fn load(language: &str) -> Self {
        let json = match language {
            "uk" => include_str!("../locales/menu.uk.json"),
            "ru" => include_str!("../locales/menu.ru.json"),
            "es" => include_str!("../locales/menu.es.json"),
            "fr" => include_str!("../locales/menu.fr.json"),
            "pl" => include_str!("../locales/menu.pl.json"),
            "sr" => include_str!("../locales/menu.sr.json"),
            "pt-BR" => include_str!("../locales/menu.pt-BR.json"),
            "zh" => include_str!("../locales/menu.zh.json"),
            "ja" => include_str!("../locales/menu.ja.json"),
            "en-PT" => include_str!("../locales/menu.en-PT.json"),
            _ => include_str!("../locales/menu.en.json"),
        };
        Self(serde_json::from_str(json).unwrap_or_default())
    }

    fn get<'a>(&'a self, key: &'a str) -> &'a str {
        self.0.get(key).map_or(key, String::as_str)
    }
}

/// The native menu bar in `language`: the usual macOS app, edit and window menus plus
/// launcher commands.
pub fn build<R: Runtime>(app: &AppHandle<R>, language: &str) -> tauri::Result<Menu<R>> {
    let text = MenuText::load(language);
    let item = |action: MenuAction| -> tauri::Result<_> {
        let (id, accelerator) = action.spec();
        let builder = MenuItemBuilder::with_id(id, text.get(id));
        match accelerator {
            Some(accelerator) => builder.accelerator(accelerator),
            None => builder,
        }
        .build(app)
    };

    let app_menu = SubmenuBuilder::new(app, "JeLauncher")
        .item(&item(MenuAction::About)?)
        .separator()
        .item(&item(MenuAction::Settings)?)
        .separator()
        .services_with_text(text.get("services"))
        .separator()
        .hide_with_text(text.get("hide"))
        .hide_others_with_text(text.get("hide-others"))
        .show_all_with_text(text.get("show-all"))
        .separator()
        .quit_with_text(text.get("quit"))
        .build()?;
    let file = SubmenuBuilder::new(app, text.get("file"))
        .item(&item(MenuAction::NewInstance)?)
        .item(&item(MenuAction::ImportInstance)?)
        .separator()
        .item(&item(MenuAction::OpenInstancesFolder)?)
        .item(&item(MenuAction::OpenLauncherFolder)?)
        .separator()
        .close_window_with_text(text.get("close-window"))
        .build()?;
    let edit = SubmenuBuilder::new(app, text.get("edit"))
        .undo_with_text(text.get("undo"))
        .redo_with_text(text.get("redo"))
        .separator()
        .cut_with_text(text.get("cut"))
        .copy_with_text(text.get("copy"))
        .paste_with_text(text.get("paste"))
        .select_all_with_text(text.get("select-all"))
        .build()?;
    let view = SubmenuBuilder::new(app, text.get("view"))
        .item(&item(MenuAction::Refresh)?)
        .item(&item(MenuAction::News)?)
        .separator()
        .fullscreen_with_text(text.get("fullscreen"))
        .build()?;
    let profiles = SubmenuBuilder::new(app, text.get("profiles"))
        .item(&item(MenuAction::ManageAccounts)?)
        .build()?;
    let window = SubmenuBuilder::new(app, text.get("window"))
        .minimize_with_text(text.get("minimize"))
        .maximize_with_text(text.get("zoom"))
        .build()?;
    let help = SubmenuBuilder::new(app, text.get("help"))
        .item(&item(MenuAction::SourceCode)?)
        .item(&item(MenuAction::Discord)?)
        .item(&item(MenuAction::LauncherLog)?)
        .build()?;
    MenuBuilder::new(app)
        .items(&[&app_menu, &file, &edit, &view, &profiles, &window, &help])
        .build()
}

/// Forwards a click on one of the launcher's own menu items to the frontend.
pub fn handle<R: Runtime>(app: &AppHandle<R>, id: &str) {
    if let Some(action) = MenuAction::ALL
        .into_iter()
        .find(|action| action.spec().0 == id)
    {
        let _ = MenuActionEvent { action }.emit(app);
    }
}
