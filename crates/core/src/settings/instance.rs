use serde::{Deserialize, Serialize};

use super::GlobalSettings;

/// Per-instance overrides; `None` falls back to the global value.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, specta::Type)]
#[serde(default, rename_all = "camelCase")]
pub struct InstanceSettings {
    pub java_path: Option<String>,
    pub min_memory_mb: Option<u32>,
    pub max_memory_mb: Option<u32>,
    pub jvm_args: Option<String>,
    pub window_width: Option<u32>,
    pub window_height: Option<u32>,
    pub window_maximized: Option<bool>,
}

/// Settings an instance actually launches with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveSettings {
    pub java_path: Option<String>,
    pub min_memory_mb: u32,
    pub max_memory_mb: u32,
    pub jvm_args: String,
    pub window_width: u32,
    pub window_height: u32,
    pub window_maximized: bool,
}

impl InstanceSettings {
    pub fn resolve(&self, global: &GlobalSettings) -> EffectiveSettings {
        EffectiveSettings {
            java_path: self.java_path.clone().or_else(|| global.java.path.clone()),
            min_memory_mb: self.min_memory_mb.unwrap_or(global.java.min_memory_mb),
            max_memory_mb: self.max_memory_mb.unwrap_or(global.java.max_memory_mb),
            jvm_args: self
                .jvm_args
                .clone()
                .unwrap_or_else(|| global.java.jvm_args.clone()),
            window_width: self.window_width.unwrap_or(global.window.width),
            window_height: self.window_height.unwrap_or(global.window.height),
            window_maximized: self.window_maximized.unwrap_or(global.window.maximized),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overrides_win_over_global() {
        let global = GlobalSettings::default();
        let instance = InstanceSettings {
            max_memory_mb: Some(2048),
            ..Default::default()
        };
        let effective = instance.resolve(&global);
        assert_eq!(effective.max_memory_mb, 2048);
        assert_eq!(effective.min_memory_mb, global.java.min_memory_mb);
        assert_eq!(effective.window_width, global.window.width);
    }
}
