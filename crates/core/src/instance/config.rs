use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::settings::InstanceSettings;
use crate::{Result, fs};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(default, rename_all = "camelCase")]
pub struct InstanceConfig {
    pub name: String,
    pub icon_key: String,
    pub notes: String,
    pub group: Option<String>,
    pub minecraft_version: String,
    /// Unix time in milliseconds; 0 when never launched.
    pub last_launch_time: i64,
    /// Seconds.
    pub total_time_played: i64,
    pub settings: InstanceSettings,
}

impl Default for InstanceConfig {
    fn default() -> Self {
        Self {
            name: "Unnamed Instance".into(),
            icon_key: "default".into(),
            notes: String::new(),
            group: None,
            minecraft_version: String::new(),
            last_launch_time: 0,
            total_time_played: 0,
            settings: InstanceSettings::default(),
        }
    }
}

impl InstanceConfig {
    pub fn load(path: &Path) -> Result<Option<Self>> {
        fs::read_json(path)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        fs::write_json(path, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_file_falls_back_to_defaults() -> std::result::Result<(), serde_json::Error> {
        let config: InstanceConfig =
            serde_json::from_str(r#"{"name":"My Pack","minecraftVersion":"1.21.4"}"#)?;
        assert_eq!(config.name, "My Pack");
        assert_eq!(config.icon_key, "default");
        assert_eq!(config.settings, InstanceSettings::default());
        Ok(())
    }
}
