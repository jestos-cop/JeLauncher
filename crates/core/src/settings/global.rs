use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::modplatform::{Provider, ReleaseType};
use crate::{Result, fs};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, specta::Type)]
#[serde(default, rename_all = "camelCase")]
pub struct GlobalSettings {
    pub java: JavaSettings,
    pub window: WindowSettings,
    pub launcher: LauncherSettings,
    pub network: NetworkSettings,
    pub commands: CommandSettings,
    pub appearance: AppearanceSettings,
    pub apis: ApiSettings,
    pub tools: ToolSettings,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(default, rename_all = "camelCase")]
pub struct ToolSettings {
    /// JProfiler installation folder.
    pub jprofiler_path: Option<String>,
    pub jprofiler_port: u16,
    /// VisualVM program; `None` looks for `jvisualvm` or `visualvm` on the PATH.
    pub visualvm_path: Option<String>,
    /// Command that opens instance files, e.g. `code`; `None` uses the system default app.
    pub editor: Option<String>,
}

impl Default for ToolSettings {
    fn default() -> Self {
        Self {
            jprofiler_path: None,
            jprofiler_port: 42042,
            visualvm_path: None,
            editor: None,
        }
    }
}

/// Credentials that replace or add to the ones the launcher was built with.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(default, rename_all = "camelCase")]
pub struct ApiSettings {
    /// Azure application id for Microsoft sign-in.
    pub microsoft_client_id: Option<String>,
    /// Modrinth personal access token, only needed for private projects.
    pub modrinth_token: Option<String>,
    pub curseforge_api_key: Option<String>,
    /// Fetches CurseForge files their authors block from Modrinth when it has the same file.
    pub modrinth_fallback_for_blocked: bool,
    /// Technic client id, only needed for private packs.
    pub technic_client_id: Option<String>,
}

impl Default for ApiSettings {
    fn default() -> Self {
        Self {
            microsoft_client_id: None,
            modrinth_token: None,
            curseforge_api_key: None,
            modrinth_fallback_for_blocked: true,
            technic_client_id: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(default, rename_all = "camelCase")]
pub struct AppearanceSettings {
    pub theme: Theme,
    /// Interface language code such as `uk`; `None` follows the system language.
    pub language: Option<String>,
    /// Shows the latest Minecraft news headline above the instances.
    pub show_news_bar: bool,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            language: None,
            show_news_bar: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(default, rename_all = "camelCase")]
pub struct JavaSettings {
    /// `None` lets the launcher pick a compatible Java automatically.
    pub path: Option<String>,
    pub min_memory_mb: u32,
    pub max_memory_mb: u32,
    pub jvm_args: String,
    /// Download Mojang's Java runtime when no installed Java fits the game.
    pub auto_download: bool,
}

impl Default for JavaSettings {
    fn default() -> Self {
        Self {
            path: None,
            min_memory_mb: 512,
            max_memory_mb: 4096,
            jvm_args: String::new(),
            auto_download: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(default, rename_all = "camelCase")]
pub struct WindowSettings {
    pub width: u32,
    pub height: u32,
    pub maximized: bool,
}

impl Default for WindowSettings {
    fn default() -> Self {
        Self {
            width: 854,
            height: 480,
            maximized: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(default, rename_all = "camelCase")]
pub struct LauncherSettings {
    pub show_console: bool,
    pub show_console_on_error: bool,
    pub auto_close_console: bool,
    pub record_game_time: bool,
    /// Set once the first-run setup has been finished or skipped.
    pub setup_completed: bool,
    /// Remembers which platform project installed mods and packs came from.
    pub track_resource_sources: bool,
    /// The least stable releases "Check for Updates" offers.
    pub update_channel: UpdateChannel,
    /// Which platform "Identify Sources" asks first; files it does not know go to the other.
    pub source_lookup_order: SourceLookupOrder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum SourceLookupOrder {
    #[default]
    ModrinthFirst,
    CurseForgeFirst,
}

impl SourceLookupOrder {
    pub fn providers(self) -> [Provider; 2] {
        match self {
            Self::ModrinthFirst => [Provider::Modrinth, Provider::CurseForge],
            Self::CurseForgeFirst => [Provider::CurseForge, Provider::Modrinth],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum UpdateChannel {
    Release,
    #[default]
    Beta,
    Alpha,
}

impl UpdateChannel {
    pub fn allows(self, release_type: ReleaseType) -> bool {
        match self {
            Self::Release => release_type == ReleaseType::Release,
            Self::Beta => release_type != ReleaseType::Alpha,
            Self::Alpha => true,
        }
    }

    /// Modrinth's names of the release types this channel allows.
    pub fn version_types(self) -> Vec<&'static str> {
        match self {
            Self::Release => vec!["release"],
            Self::Beta => vec!["release", "beta"],
            Self::Alpha => vec!["release", "beta", "alpha"],
        }
    }
}

impl Default for LauncherSettings {
    fn default() -> Self {
        Self {
            show_console: false,
            show_console_on_error: true,
            auto_close_console: false,
            record_game_time: true,
            setup_completed: false,
            track_resource_sources: true,
            update_channel: UpdateChannel::default(),
            source_lookup_order: SourceLookupOrder::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(default, rename_all = "camelCase")]
pub struct NetworkSettings {
    pub concurrent_downloads: u32,
    pub request_timeout_secs: u32,
    pub retries: u32,
    /// `None` uses the system proxy settings.
    pub proxy: Option<String>,
}

impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
            concurrent_downloads: 6,
            request_timeout_secs: 60,
            retries: 1,
            proxy: None,
        }
    }
}

/// Commands run around the game and extra environment variables for it.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, specta::Type)]
#[serde(default, rename_all = "camelCase")]
pub struct CommandSettings {
    pub pre_launch: String,
    /// Prefix for the Java invocation, e.g. `mangohud` or `prime-run`.
    pub wrapper: String,
    pub post_exit: String,
    pub environment: BTreeMap<String, String>,
}

impl GlobalSettings {
    pub fn load(path: &Path) -> Result<Self> {
        Ok(fs::read_json(path)?.unwrap_or_default())
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        fs::write_json(path, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_fields_take_defaults() -> std::result::Result<(), serde_json::Error> {
        let settings: GlobalSettings = serde_json::from_str(r#"{"java":{"maxMemoryMb":8192}}"#)?;
        assert_eq!(settings.java.max_memory_mb, 8192);
        assert_eq!(settings.java.min_memory_mb, 512);
        assert_eq!(settings.window, WindowSettings::default());
        Ok(())
    }
}
