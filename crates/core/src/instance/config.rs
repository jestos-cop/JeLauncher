use std::path::Path;

use ini::{Ini, Properties};

use crate::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceConfig {
    pub name: String,
    pub icon_key: String,
    pub instance_type: String,
    pub notes: String,
    pub last_launch_time: i64,
    pub total_time_played: i64,
}

impl Default for InstanceConfig {
    fn default() -> Self {
        Self {
            name: "Unnamed Instance".into(),
            icon_key: "default".into(),
            instance_type: String::new(),
            notes: String::new(),
            last_launch_time: 0,
            total_time_played: 0,
        }
    }
}

impl InstanceConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let ini = Ini::load_from_file(path).map_err(|err| match err {
            ini::Error::Io(source) => Error::io(path, source),
            ini::Error::Parse(parse) => Error::Ini {
                path: path.to_path_buf(),
                message: parse.to_string(),
            },
        })?;
        Ok(Self::from_ini(&ini))
    }

    /// QSettings writes keys under `[General]`; older files have no section at all.
    fn from_ini(ini: &Ini) -> Self {
        let props = ini
            .section(Some("General"))
            .unwrap_or_else(|| ini.general_section());
        Self::from_properties(props)
    }

    fn from_properties(props: &Properties) -> Self {
        let defaults = Self::default();
        let text = |key: &str, default: String| props.get(key).map_or(default, str::to_owned);
        let number = |key: &str| props.get(key).and_then(|v| v.parse().ok()).unwrap_or(0);

        Self {
            name: text("name", defaults.name),
            icon_key: text("iconKey", defaults.icon_key),
            instance_type: text("InstanceType", defaults.instance_type),
            notes: text("notes", defaults.notes),
            last_launch_time: number("lastLaunchTime"),
            total_time_played: number("totalTimePlayed").max(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(source: &str) -> InstanceConfig {
        let ini = Ini::load_from_str(source).unwrap_or_default();
        InstanceConfig::from_ini(&ini)
    }

    #[test]
    fn reads_general_section() {
        let config = parse(
            "[General]\nname=My Pack\niconKey=flame\nInstanceType=OneSix\nlastLaunchTime=1700000000000\ntotalTimePlayed=3600\n",
        );
        assert_eq!(config.name, "My Pack");
        assert_eq!(config.icon_key, "flame");
        assert_eq!(config.instance_type, "OneSix");
        assert_eq!(config.last_launch_time, 1_700_000_000_000);
        assert_eq!(config.total_time_played, 3600);
    }

    #[test]
    fn reads_legacy_sectionless_file() {
        assert_eq!(parse("name=Old Instance\n").name, "Old Instance");
    }

    #[test]
    fn falls_back_to_defaults() {
        assert_eq!(parse(""), InstanceConfig::default());
    }

    #[test]
    fn negative_play_time_is_reset() {
        assert_eq!(
            parse("[General]\ntotalTimePlayed=-5\n").total_time_played,
            0
        );
    }
}
