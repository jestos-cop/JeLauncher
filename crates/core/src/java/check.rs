use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::process::Command;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct JavaInstall {
    pub path: String,
    pub version: String,
    pub major: u32,
    pub arch: String,
    pub vendor: String,
}

impl JavaInstall {
    pub fn is_native_arch(&self) -> bool {
        let host = std::env::consts::ARCH;
        self.arch == host || (host == "x86_64" && self.arch == "amd64")
    }
}

const CHECK_TIMEOUT: Duration = Duration::from_secs(15);

/// Runs the binary and reads its system properties; `None` if it is not a working Java.
pub async fn check_java(path: &Path) -> Option<JavaInstall> {
    let output = Command::new(path)
        .args(["-XshowSettings:properties", "-version"])
        .stdin(Stdio::null())
        .kill_on_drop(true)
        .output();
    let output = tokio::time::timeout(CHECK_TIMEOUT, output)
        .await
        .ok()?
        .ok()?;
    // The properties are printed to stderr.
    parse_properties(path, &String::from_utf8_lossy(&output.stderr))
}

fn parse_properties(path: &Path, text: &str) -> Option<JavaInstall> {
    let property = |key: &str| {
        text.lines()
            .filter_map(|line| line.trim().split_once(" = "))
            .find(|(name, _)| *name == key)
            .map(|(_, value)| value.trim().to_owned())
    };
    let version = property("java.version")?;
    Some(JavaInstall {
        path: path.to_string_lossy().into_owned(),
        major: parse_major_version(&version)?,
        version,
        arch: property("os.arch").unwrap_or_default(),
        vendor: property("java.vendor").unwrap_or_default(),
    })
}

/// `1.8.0_392` → 8, `21.0.11` → 21, `25` → 25.
pub fn parse_major_version(version: &str) -> Option<u32> {
    let mut parts = version
        .split(|c: char| !c.is_ascii_digit())
        .filter(|p| !p.is_empty());
    let first: u32 = parts.next()?.parse().ok()?;
    if first == 1 {
        parts.next()?.parse().ok()
    } else {
        Some(first)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn major_versions() {
        assert_eq!(parse_major_version("1.8.0_392"), Some(8));
        assert_eq!(parse_major_version("21.0.11"), Some(21));
        assert_eq!(parse_major_version("25"), Some(25));
        assert_eq!(parse_major_version("17-ea"), Some(17));
        assert_eq!(parse_major_version(""), None);
    }

    #[test]
    fn parses_show_settings_output() {
        let text = "Property settings:\n    java.vendor = Eclipse Adoptium\n    java.version = 21.0.11\n    os.arch = aarch64\n";
        let install = parse_properties(Path::new("/bin/java"), text);
        assert_eq!(
            install,
            Some(JavaInstall {
                path: "/bin/java".into(),
                version: "21.0.11".into(),
                major: 21,
                arch: "aarch64".into(),
                vendor: "Eclipse Adoptium".into(),
            })
        );
    }
}
