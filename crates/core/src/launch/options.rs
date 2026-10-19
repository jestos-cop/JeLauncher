use serde::Deserialize;

use crate::minecraft::{Features, VersionJson};

/// Where the game should go right after starting.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, specta::Type)]
#[serde(tag = "type", content = "target", rename_all = "camelCase")]
pub enum QuickPlay {
    /// A server address, `host[:port]`.
    Server(String),
    /// A world folder name inside `saves`.
    World(String),
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, specta::Type)]
#[serde(default, rename_all = "camelCase")]
pub struct LaunchOptions {
    pub quick_play: Option<QuickPlay>,
    pub demo: bool,
}

impl LaunchOptions {
    pub fn features(&self, version: &VersionJson, custom_resolution: bool) -> Features {
        let quick_play = self
            .quick_play
            .as_ref()
            .filter(|_| supports_quick_play(version));
        Features {
            custom_resolution,
            demo_user: self.demo,
            quick_play_singleplayer: matches!(quick_play, Some(QuickPlay::World(_))),
            quick_play_multiplayer: matches!(quick_play, Some(QuickPlay::Server(_))),
        }
    }
}

/// Versions since 1.20 declare quick play arguments; older ones only know `--server`.
pub fn supports_quick_play(version: &VersionJson) -> bool {
    version.arguments.as_ref().is_some_and(|arguments| {
        arguments
            .game
            .iter()
            .flat_map(|argument| argument.all_values())
            .any(|value| value.contains("${quickPlayMultiplayer}"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn version(game_args: &str) -> Result<VersionJson, serde_json::Error> {
        serde_json::from_str(&format!(
            r#"{{"id":"1.21.4","mainClass":"M","assets":"19",
                "assetIndex":{{"id":"19","sha1":"s","size":1,"url":"u"}},
                "downloads":{{"client":{{"sha1":"s","size":1,"url":"u"}}}},
                "arguments":{{"game":{game_args},"jvm":[]}}}}"#
        ))
    }

    #[test]
    fn enables_quick_play_features_only_when_the_version_supports_them()
    -> Result<(), serde_json::Error> {
        let modern = version(
            r#"[{"rules":[{"action":"allow","features":{"is_quick_play_multiplayer":true}}],
                 "value":["--quickPlayMultiplayer","${quickPlayMultiplayer}"]}]"#,
        )?;
        let legacy = version("[]")?;
        let options = LaunchOptions {
            quick_play: Some(QuickPlay::Server("mc.example.com".into())),
            demo: false,
        };
        assert!(options.features(&modern, false).quick_play_multiplayer);
        assert!(!options.features(&legacy, false).quick_play_multiplayer);
        Ok(())
    }
}
