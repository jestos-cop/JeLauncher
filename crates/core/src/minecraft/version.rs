use std::collections::HashMap;

use serde::Deserialize;

use super::rule::{Environment, Rule};

/// A Mojang version file (`<version>.json`).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionJson {
    pub id: String,
    pub main_class: String,
    #[serde(default)]
    pub arguments: Option<Arguments>,
    /// Pre-1.13 versions pass game arguments as a single string.
    #[serde(default)]
    pub minecraft_arguments: Option<String>,
    #[serde(default)]
    pub libraries: Vec<Library>,
    pub asset_index: AssetIndexRef,
    pub assets: String,
    pub downloads: VersionDownloads,
    #[serde(default)]
    pub java_version: Option<JavaVersion>,
    #[serde(default)]
    pub logging: Option<Logging>,
    #[serde(rename = "type", default)]
    pub kind: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Arguments {
    #[serde(default)]
    pub game: Vec<Argument>,
    #[serde(default)]
    pub jvm: Vec<Argument>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum Argument {
    Plain(String),
    Conditional {
        rules: Vec<Rule>,
        value: ArgumentValue,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum ArgumentValue {
    One(String),
    Many(Vec<String>),
}

impl Argument {
    /// Raw (unsubstituted) values this argument contributes in `env`.
    pub fn values(&self, env: &Environment) -> Vec<&str> {
        match self {
            Self::Plain(value) => vec![value.as_str()],
            Self::Conditional { rules, value } if env.allows(rules) => match value {
                ArgumentValue::One(value) => vec![value.as_str()],
                ArgumentValue::Many(values) => values.iter().map(String::as_str).collect(),
            },
            Self::Conditional { .. } => Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Artifact {
    #[serde(default)]
    pub path: Option<String>,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VersionDownloads {
    pub client: Artifact,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetIndexRef {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaVersion {
    pub major_version: u32,
    /// Mojang runtime that provides this Java, e.g. `java-runtime-delta`.
    #[serde(default)]
    pub component: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Logging {
    pub client: Option<LoggingConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    pub argument: String,
    pub file: LoggingFile,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingFile {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Library {
    pub name: String,
    #[serde(default)]
    pub downloads: Option<LibraryDownloads>,
    /// Legacy native classifiers per OS, e.g. `{"osx": "natives-osx"}`.
    #[serde(default)]
    pub natives: Option<HashMap<String, String>>,
    #[serde(default)]
    pub rules: Vec<Rule>,
    /// Maven repository base URL for libraries without explicit downloads.
    #[serde(default)]
    pub url: Option<String>,
    /// Loader profiles put the checksum next to the Maven coordinates.
    #[serde(default)]
    pub sha1: Option<String>,
    #[serde(default)]
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct LibraryDownloads {
    #[serde(default)]
    pub artifact: Option<Artifact>,
    #[serde(default)]
    pub classifiers: Option<HashMap<String, Artifact>>,
}

const DEFAULT_MAVEN: &str = "https://libraries.minecraft.net/";

impl Library {
    /// The jar that goes on the classpath, if any.
    pub fn artifact(&self) -> Option<Artifact> {
        if let Some(artifact) = self.downloads.as_ref().and_then(|d| d.artifact.clone()) {
            let path = artifact
                .path
                .clone()
                .or_else(|| maven_path(&self.name, None));
            return Some(Artifact { path, ..artifact });
        }
        if self.natives.is_some() {
            return None;
        }
        let path = maven_path(&self.name, None)?;
        let base = self.url.as_deref().unwrap_or(DEFAULT_MAVEN);
        Some(Artifact {
            url: format!("{}/{path}", base.trim_end_matches('/')),
            path: Some(path),
            sha1: self.sha1.clone().unwrap_or_default(),
            size: self.size.unwrap_or_default(),
        })
    }

    /// `group:artifact[:classifier]`, used to let one library replace another of a different version.
    pub fn identity(&self) -> String {
        let parts: Vec<&str> = self.name.split(':').collect();
        match parts.as_slice() {
            [group, artifact, _, classifier, ..] => format!("{group}:{artifact}:{classifier}"),
            [group, artifact, ..] => format!("{group}:{artifact}"),
            _ => self.name.clone(),
        }
    }

    /// Legacy native jar that must be extracted before launch.
    pub fn native_artifact(&self, env: &Environment) -> Option<Artifact> {
        let classifier = self
            .natives
            .as_ref()?
            .get(env.os.mojang_name())?
            .replace("${arch}", if env.arch == "x86" { "32" } else { "64" });
        let artifact = self
            .downloads
            .as_ref()?
            .classifiers
            .as_ref()?
            .get(&classifier)?
            .clone();
        let path = artifact
            .path
            .clone()
            .or_else(|| maven_path(&self.name, Some(&classifier)));
        Some(Artifact { path, ..artifact })
    }
}

/// Converts `group:artifact:version[:classifier]` into a repository path.
pub fn maven_path(name: &str, classifier: Option<&str>) -> Option<String> {
    let (name, extension) = name.split_once('@').unwrap_or((name, "jar"));
    let mut parts = name.split(':');
    let group = parts.next()?;
    let artifact = parts.next()?;
    let version = parts.next()?;
    let classifier = classifier.or(parts.next());
    let file = match classifier {
        Some(classifier) => format!("{artifact}-{version}-{classifier}.{extension}"),
        None => format!("{artifact}-{version}.{extension}"),
    };
    Some(format!(
        "{}/{artifact}/{version}/{file}",
        group.replace('.', "/")
    ))
}

#[cfg(test)]
mod tests {
    use super::super::rule::{Features, OsName};
    use super::*;

    #[test]
    fn maven_paths() {
        assert_eq!(
            maven_path("org.lwjgl:lwjgl:3.3.3:natives-macos-arm64", None).as_deref(),
            Some("org/lwjgl/lwjgl/3.3.3/lwjgl-3.3.3-natives-macos-arm64.jar")
        );
        assert_eq!(
            maven_path("net.fabricmc:fabric-loader:0.16.9", None).as_deref(),
            Some("net/fabricmc/fabric-loader/0.16.9/fabric-loader-0.16.9.jar")
        );
        assert_eq!(
            maven_path("a:b:1@zip", None).as_deref(),
            Some("a/b/1/b-1.zip")
        );
        assert_eq!(
            maven_path("net.minecraft:client:1.21.4:mappings@txt", None).as_deref(),
            Some("net/minecraft/client/1.21.4/client-1.21.4-mappings.txt")
        );
    }

    #[test]
    fn conditional_arguments_follow_rules() -> Result<(), serde_json::Error> {
        let args: Vec<Argument> = serde_json::from_str(
            r#"["--a", {"rules":[{"action":"allow","os":{"name":"osx"}}],"value":["-XstartOnFirstThread"]}]"#,
        )?;
        let mac = Environment {
            os: OsName::MacOs,
            arch: "aarch64",
            features: Features::default(),
        };
        let linux = Environment {
            os: OsName::Linux,
            ..mac.clone()
        };
        let collect =
            |env: &Environment| args.iter().flat_map(|a| a.values(env)).collect::<Vec<_>>();
        assert_eq!(collect(&mac), ["--a", "-XstartOnFirstThread"]);
        assert_eq!(collect(&linux), ["--a"]);
        Ok(())
    }

    #[test]
    fn legacy_natives_use_classifier() -> Result<(), serde_json::Error> {
        let library: Library = serde_json::from_str(
            r#"{"name":"org.lwjgl.lwjgl:lwjgl-platform:2.9.4","natives":{"osx":"natives-osx","windows":"natives-windows-${arch}"},
                "downloads":{"classifiers":{
                    "natives-osx":{"path":"p/osx.jar","sha1":"a","size":1,"url":"u1"},
                    "natives-windows-64":{"path":"p/win.jar","sha1":"b","size":2,"url":"u2"}}}}"#,
        )?;
        let windows = Environment {
            os: OsName::Windows,
            arch: "x86_64",
            features: Features::default(),
        };
        assert_eq!(
            library.native_artifact(&windows).map(|a| a.url),
            Some("u2".into())
        );
        assert!(library.artifact().is_none());
        Ok(())
    }
}
