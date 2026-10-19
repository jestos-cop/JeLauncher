use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Rule {
    pub action: RuleAction,
    #[serde(default)]
    pub os: Option<OsRule>,
    #[serde(default)]
    pub features: Option<HashMap<String, bool>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleAction {
    Allow,
    Disallow,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct OsRule {
    pub name: Option<String>,
    pub arch: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsName {
    Windows,
    MacOs,
    Linux,
}

impl OsName {
    pub fn current() -> Self {
        if cfg!(target_os = "windows") {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::MacOs
        } else {
            Self::Linux
        }
    }

    /// Name used in Mojang version rules and legacy `natives` maps.
    pub fn mojang_name(self) -> &'static str {
        match self {
            Self::Windows => "windows",
            Self::MacOs => "osx",
            Self::Linux => "linux",
        }
    }
}

/// Launch features that conditional arguments can depend on.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Features {
    pub custom_resolution: bool,
    pub demo_user: bool,
    pub quick_play_singleplayer: bool,
    pub quick_play_multiplayer: bool,
}

impl Features {
    fn get(&self, name: &str) -> bool {
        match name {
            "has_custom_resolution" => self.custom_resolution,
            "is_demo_user" => self.demo_user,
            "has_quick_plays_support" => {
                self.quick_play_singleplayer || self.quick_play_multiplayer
            }
            "is_quick_play_singleplayer" => self.quick_play_singleplayer,
            "is_quick_play_multiplayer" => self.quick_play_multiplayer,
            _ => false,
        }
    }
}

/// Platform and feature set that rules are evaluated against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Environment {
    pub os: OsName,
    pub arch: &'static str,
    pub features: Features,
}

impl Environment {
    pub fn current(features: Features) -> Self {
        Self {
            os: OsName::current(),
            arch: std::env::consts::ARCH,
            features,
        }
    }

    /// Applies rules the way the official launcher does: the last matching rule wins
    /// and an empty rule list allows everything.
    pub fn allows(&self, rules: &[Rule]) -> bool {
        if rules.is_empty() {
            return true;
        }
        rules
            .iter()
            .rev()
            .find(|rule| self.matches(rule))
            .is_some_and(|rule| rule.action == RuleAction::Allow)
    }

    fn matches(&self, rule: &Rule) -> bool {
        let os_matches = rule.os.as_ref().is_none_or(|os| {
            os.name
                .as_deref()
                .is_none_or(|name| name == self.os.mojang_name())
                && os
                    .arch
                    .as_deref()
                    .is_none_or(|arch| self.arch_matches(arch))
        });
        let features_match = rule.features.as_ref().is_none_or(|features| {
            features
                .iter()
                .all(|(name, &value)| self.features.get(name) == value)
        });
        os_matches && features_match
    }

    fn arch_matches(&self, arch: &str) -> bool {
        match arch {
            "x86" => self.arch == "x86",
            "x86_64" => self.arch == "x86_64",
            "arm64" | "aarch64" => self.arch == "aarch64",
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules(json: &str) -> Vec<Rule> {
        serde_json::from_str(json).unwrap_or_default()
    }

    fn env(os: OsName, arch: &'static str) -> Environment {
        Environment {
            os,
            arch,
            features: Features::default(),
        }
    }

    #[test]
    fn last_matching_rule_wins() {
        let rules = rules(r#"[{"action":"allow"},{"action":"disallow","os":{"name":"osx"}}]"#);
        assert!(env(OsName::Linux, "x86_64").allows(&rules));
        assert!(!env(OsName::MacOs, "aarch64").allows(&rules));
    }

    #[test]
    fn allow_only_for_listed_os() {
        let rules = rules(r#"[{"action":"allow","os":{"name":"osx"}}]"#);
        assert!(env(OsName::MacOs, "aarch64").allows(&rules));
        assert!(!env(OsName::Windows, "x86_64").allows(&rules));
    }

    #[test]
    fn feature_rules_follow_features() {
        let rules = rules(r#"[{"action":"allow","features":{"has_custom_resolution":true}}]"#);
        let mut environment = env(OsName::Linux, "x86_64");
        assert!(!environment.allows(&rules));
        environment.features.custom_resolution = true;
        assert!(environment.allows(&rules));
    }

    #[test]
    fn x86_means_32_bit() {
        let rules = rules(r#"[{"action":"allow","os":{"arch":"x86"}}]"#);
        assert!(!env(OsName::Windows, "x86_64").allows(&rules));
        assert!(env(OsName::Windows, "x86").allows(&rules));
    }
}
