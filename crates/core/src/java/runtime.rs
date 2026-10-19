use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::archive::safe_relative_path;
use crate::net::{Download, DownloadProgress, HttpClient};
use crate::{Error, Result};

const RUNTIMES_URL: &str = "https://launchermeta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json";

/// Used by versions that predate the `javaVersion` field.
pub const LEGACY_COMPONENT: &str = "jre-legacy";

#[derive(Deserialize)]
struct RuntimeEntry {
    manifest: ManifestRef,
}

#[derive(Deserialize)]
struct ManifestRef {
    sha1: String,
    url: String,
}

#[derive(Deserialize)]
struct RuntimeManifest {
    files: HashMap<String, RuntimeFile>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum RuntimeFile {
    Directory,
    File {
        downloads: FileDownloads,
        #[serde(default)]
        executable: bool,
    },
    Link {
        target: String,
    },
}

#[derive(Deserialize)]
struct FileDownloads {
    raw: RawDownload,
}

#[derive(Deserialize)]
struct RawDownload {
    sha1: String,
    size: u64,
    url: String,
}

/// Mojang's platform keys for the running system, best first. ARM machines fall back to
/// x64 builds, which run under emulation; old runtimes like `jre-legacy` only exist as those.
fn platform_keys() -> &'static [&'static str] {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => &["mac-os-arm64", "mac-os"],
        ("macos", _) => &["mac-os"],
        ("windows", "aarch64") => &["windows-arm64", "windows-x64"],
        ("windows", "x86") => &["windows-x86"],
        ("windows", _) => &["windows-x64"],
        ("linux", "x86") => &["linux-i386"],
        ("linux", "x86_64") => &["linux"],
        _ => &[],
    }
}

/// Path of the Java binary inside an installed runtime component.
pub fn runtime_java(component_dir: &Path) -> PathBuf {
    let home = if cfg!(target_os = "macos") {
        component_dir.join("jre.bundle/Contents/Home")
    } else {
        component_dir.to_path_buf()
    };
    home.join("bin")
        .join(if cfg!(windows) { "javaw.exe" } else { "java" })
}

/// Downloads Mojang's Java runtime `component` into `java_dir/<component>` and
/// returns the Java binary. Files already present with the right hash are kept.
pub async fn install_runtime(
    http: &HttpClient,
    component: &str,
    java_dir: &Path,
    concurrency: usize,
    on_progress: impl Fn(DownloadProgress),
) -> Result<PathBuf> {
    let mut runtimes: HashMap<String, HashMap<String, Vec<RuntimeEntry>>> =
        http.get_json(RUNTIMES_URL).await?;
    let (platform, entry) = platform_keys()
        .iter()
        .find_map(|platform| {
            let entry = runtimes
                .get_mut(*platform)?
                .remove(component)?
                .into_iter()
                .next()?;
            Some((*platform, entry))
        })
        .ok_or_else(|| {
            Error::InvalidInput(format!(
                "Java runtime {component} is not available for this platform"
            ))
        })?;
    log::info!("downloading Java runtime {component} for {platform}");

    let manifest_path = java_dir.join(format!("{component}.manifest.json"));
    http.download(&Download {
        url: entry.manifest.url,
        path: manifest_path.clone(),
        sha1: Some(entry.manifest.sha1),
        size: None,
    })
    .await?;
    let manifest: RuntimeManifest = crate::fs::read_json(&manifest_path)?
        .ok_or_else(|| Error::InvalidInput(format!("missing manifest for {component}")))?;

    let root = java_dir.join(component);
    install_files(http, &manifest, &root, concurrency, on_progress).await?;
    Ok(runtime_java(&root))
}

async fn install_files(
    http: &HttpClient,
    manifest: &RuntimeManifest,
    root: &Path,
    concurrency: usize,
    on_progress: impl Fn(DownloadProgress),
) -> Result<()> {
    let mut downloads = Vec::new();
    let mut executables = Vec::new();
    let mut links = Vec::new();
    for (name, file) in &manifest.files {
        let relative = safe_relative_path(name)
            .ok_or_else(|| Error::InvalidInput(format!("unsafe path in Java runtime: {name}")))?;
        let path = root.join(relative);
        match file {
            RuntimeFile::Directory => {
                std::fs::create_dir_all(&path).map_err(|err| Error::io(&path, err))?
            }
            RuntimeFile::File {
                downloads: d,
                executable,
            } => {
                if *executable {
                    executables.push(path.clone());
                }
                downloads.push(Download {
                    url: d.raw.url.clone(),
                    path,
                    sha1: Some(d.raw.sha1.clone()),
                    size: Some(d.raw.size),
                });
            }
            RuntimeFile::Link { target } => links.push((path, target.clone())),
        }
    }
    http.download_all(downloads, concurrency, on_progress)
        .await?;
    mark_executable(&executables)?;
    create_links(&links)
}

#[cfg(unix)]
fn mark_executable(paths: &[PathBuf]) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    for path in paths {
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
            .map_err(|err| Error::io(path, err))?;
    }
    Ok(())
}

#[cfg(not(unix))]
fn mark_executable(_paths: &[PathBuf]) -> Result<()> {
    Ok(())
}

#[cfg(unix)]
fn create_links(links: &[(PathBuf, String)]) -> Result<()> {
    for (path, target) in links {
        if path.symlink_metadata().is_ok() {
            continue;
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|err| Error::io(parent, err))?;
        }
        std::os::unix::fs::symlink(target, path).map_err(|err| Error::io(path, err))?;
    }
    Ok(())
}

/// Windows runtimes do not ship links that are needed to run Java.
#[cfg(not(unix))]
fn create_links(_links: &[(PathBuf, String)]) -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_runtime_manifest_entries() -> std::result::Result<(), serde_json::Error> {
        let manifest: RuntimeManifest = serde_json::from_str(
            r#"{"files":{
                "jre.bundle":{"type":"directory"},
                "jre.bundle/Contents/Home/bin/java":{"type":"file","executable":true,
                    "downloads":{"raw":{"sha1":"a","size":1,"url":"u"}}},
                "jre.bundle/Contents/Home/legal/x":{"type":"link","target":"../y"}}}"#,
        )?;
        assert_eq!(manifest.files.len(), 3);
        assert!(matches!(
            manifest.files.get("jre.bundle/Contents/Home/bin/java"),
            Some(RuntimeFile::File {
                executable: true,
                ..
            })
        ));
        Ok(())
    }
}
