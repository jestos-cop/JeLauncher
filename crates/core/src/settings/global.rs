use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{Result, fs};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, specta::Type)]
#[serde(default, rename_all = "camelCase")]
pub struct GlobalSettings {
    pub java: JavaSettings,
    pub window: WindowSettings,
    pub launcher: LauncherSettings,
    pub network: NetworkSettings,
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
}

impl Default for LauncherSettings {
    fn default() -> Self {
        Self {
            show_console: false,
            show_console_on_error: true,
            auto_close_console: false,
            record_game_time: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(default, rename_all = "camelCase")]
pub struct NetworkSettings {
    pub concurrent_downloads: u32,
    pub request_timeout_secs: u32,
    pub retries: u32,
}

impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
            concurrent_downloads: 6,
            request_timeout_secs: 60,
            retries: 1,
        }
    }
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
