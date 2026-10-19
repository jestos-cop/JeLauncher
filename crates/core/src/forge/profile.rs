use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::minecraft::{Library, maven_path};
use crate::{Error, Result};

/// `install_profile.json` of a Forge/NeoForge installer (spec 1).
#[derive(Debug, Deserialize)]
pub struct InstallProfile {
    #[serde(default = "default_version_json")]
    pub json: String,
    #[serde(default)]
    pub libraries: Vec<Library>,
    #[serde(default)]
    pub processors: Vec<Processor>,
    #[serde(default)]
    pub data: HashMap<String, SidedValue>,
}

fn default_version_json() -> String {
    "/version.json".into()
}

#[derive(Debug, Deserialize)]
pub struct SidedValue {
    pub client: String,
}

#[derive(Debug, Deserialize)]
pub struct Processor {
    pub jar: String,
    #[serde(default)]
    pub classpath: Vec<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub outputs: HashMap<String, String>,
    #[serde(default)]
    pub sides: Option<Vec<String>>,
}

impl Processor {
    pub fn runs_on_client(&self) -> bool {
        self.sides
            .as_ref()
            .is_none_or(|sides| sides.iter().any(|side| side == "client"))
    }
}

/// Resolves the placeholders used by installer data and processor arguments.
pub struct Variables {
    libraries_dir: PathBuf,
    values: HashMap<String, String>,
}

impl Variables {
    pub fn new(libraries_dir: &Path, values: HashMap<String, String>) -> Self {
        Self {
            libraries_dir: libraries_dir.to_path_buf(),
            values,
        }
    }

    pub fn insert(&mut self, key: &str, value: String) {
        self.values.insert(key.to_owned(), value);
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }

    /// `[maven:coordinates]` becomes a library path; `{KEY}` references are substituted.
    pub fn resolve(&self, value: &str) -> Result<String> {
        if let Some(coordinates) = value.strip_prefix('[').and_then(|v| v.strip_suffix(']')) {
            return Ok(self.library(coordinates)?.to_string_lossy().into_owned());
        }
        let mut resolved = String::with_capacity(value.len());
        let mut rest = value;
        while let Some(start) = rest.find('{') {
            let Some(end) = rest[start..].find('}') else {
                break;
            };
            let key = &rest[start + 1..start + end];
            let substitution = self.get(key).ok_or_else(|| {
                Error::InvalidInput(format!("installer uses unknown value {{{key}}}"))
            })?;
            resolved.push_str(&rest[..start]);
            resolved.push_str(substitution);
            rest = &rest[start + end + 1..];
        }
        resolved.push_str(rest);
        Ok(resolved)
    }

    pub fn library(&self, coordinates: &str) -> Result<PathBuf> {
        maven_path(coordinates, None)
            .map(|path| self.libraries_dir.join(path))
            .ok_or_else(|| Error::InvalidInput(format!("invalid library name {coordinates}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_libraries_and_variables() -> Result<()> {
        let vars = Variables::new(
            Path::new("/libs"),
            HashMap::from([
                ("SIDE".to_owned(), "client".to_owned()),
                ("ROOT".to_owned(), "/tmp/x".to_owned()),
            ]),
        );
        assert_eq!(
            vars.resolve("[net.minecraft:client:1.21.4:mappings@txt]")?,
            Path::new("/libs")
                .join("net/minecraft/client/1.21.4/client-1.21.4-mappings.txt")
                .to_string_lossy()
        );
        assert_eq!(vars.resolve("{ROOT}/run.sh")?, "/tmp/x/run.sh");
        assert_eq!(vars.resolve("--side")?, "--side");
        assert!(vars.resolve("{MISSING}").is_err());
        Ok(())
    }

    #[test]
    fn processors_without_sides_run_everywhere() -> std::result::Result<(), serde_json::Error> {
        let processors: Vec<Processor> = serde_json::from_str(
            r#"[{"jar":"a:b:1"},{"jar":"a:b:1","sides":["server"]},{"jar":"a:b:1","sides":["client"]}]"#,
        )?;
        let client: Vec<_> = processors.iter().map(Processor::runs_on_client).collect();
        assert_eq!(client, [true, false, true]);
        Ok(())
    }
}
