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
    LauncherLog,
}

impl MenuAction {
    const ALL: [Self; 11] = [
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
        Self::LauncherLog,
    ];

    /// Menu item id, label and keyboard shortcut.
    fn spec(self) -> (&'static str, &'static str, Option<&'static str>) {
        match self {
            Self::About => ("about", "About JeLauncher", None),
            Self::Settings => ("settings", "Settings…", Some("CmdOrCtrl+,")),
            Self::NewInstance => ("new-instance", "New Instance…", Some("CmdOrCtrl+N")),
            Self::ImportInstance => ("import-instance", "Import…", Some("CmdOrCtrl+Shift+I")),
            Self::Refresh => ("refresh", "Refresh Instances", Some("CmdOrCtrl+R")),
            Self::OpenInstancesFolder => ("open-instances-folder", "Open Instances Folder", None),
            Self::OpenLauncherFolder => ("open-launcher-folder", "Open Launcher Folder", None),
            Self::ManageAccounts => ("manage-accounts", "Manage Accounts…", None),
            Self::News => ("news", "Minecraft News", None),
            Self::SourceCode => ("source-code", "Source Code and Issues", None),
            Self::LauncherLog => ("launcher-log", "Open Launcher Log", None),
        }
    }
}

#[derive(Debug, Clone, Serialize, specta::Type, tauri_specta::Event)]
pub struct MenuActionEvent {
    pub action: MenuAction,
}

/// The native menu bar: the usual macOS app, edit and window menus plus launcher commands.
pub fn build<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    let item = |action: MenuAction| -> tauri::Result<_> {
        let (id, label, accelerator) = action.spec();
        let builder = MenuItemBuilder::with_id(id, label);
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
        .services()
        .separator()
        .hide()
        .hide_others()
        .show_all()
        .separator()
        .quit()
        .build()?;
    let file = SubmenuBuilder::new(app, "File")
        .item(&item(MenuAction::NewInstance)?)
        .item(&item(MenuAction::ImportInstance)?)
        .separator()
        .item(&item(MenuAction::OpenInstancesFolder)?)
        .item(&item(MenuAction::OpenLauncherFolder)?)
        .separator()
        .close_window()
        .build()?;
    let edit = SubmenuBuilder::new(app, "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .build()?;
    let view = SubmenuBuilder::new(app, "View")
        .item(&item(MenuAction::Refresh)?)
        .item(&item(MenuAction::News)?)
        .separator()
        .fullscreen()
        .build()?;
    let profiles = SubmenuBuilder::new(app, "Profiles")
        .item(&item(MenuAction::ManageAccounts)?)
        .build()?;
    let window = SubmenuBuilder::new(app, "Window")
        .minimize()
        .maximize()
        .build()?;
    let help = SubmenuBuilder::new(app, "Help")
        .item(&item(MenuAction::SourceCode)?)
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
