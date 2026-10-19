use std::collections::HashSet;
use std::env;
use std::path::{Path, PathBuf};

use futures_util::future::join_all;

use super::{JavaInstall, check_java};

const JAVA_BINARY: &str = if cfg!(windows) { "javaw.exe" } else { "java" };

/// Finds and validates every Java on the system plus those in `managed_dir`.
pub async fn detect_java_installs(managed_dir: &Path) -> Vec<JavaInstall> {
    let checks = candidate_paths(managed_dir)
        .into_iter()
        .map(|path| async move { check_java(&path).await });
    let mut installs: Vec<_> = join_all(checks).await.into_iter().flatten().collect();
    installs.sort_by(|a, b| b.major.cmp(&a.major).then_with(|| a.path.cmp(&b.path)));
    installs
}

/// Existing Java binaries in well-known locations, deduplicated by canonical path.
pub fn candidate_paths(managed_dir: &Path) -> Vec<PathBuf> {
    let home = env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" }).map(PathBuf::from);
    let mut homes: Vec<PathBuf> = Vec::new();

    homes.extend(java_homes_in(managed_dir));
    if let Some(java_home) = env::var_os("JAVA_HOME") {
        homes.push(PathBuf::from(java_home));
    }
    if let Some(home) = &home {
        homes.extend(java_homes_in(&home.join(".jdks")));
        homes.extend(java_homes_in(&home.join(".gradle/jdks")));
        let sdkman = env::var_os("SDKMAN_DIR").map_or_else(|| home.join(".sdkman"), PathBuf::from);
        homes.extend(java_homes_in(&sdkman.join("candidates/java")));
    }
    homes.extend(platform_homes(home.as_deref()));

    let mut binaries: Vec<PathBuf> = homes
        .iter()
        .map(|h| h.join("bin").join(JAVA_BINARY))
        .collect();
    binaries.extend(binaries_on_path());

    let mut seen = HashSet::new();
    binaries
        .into_iter()
        .filter(|path| path.is_file())
        .filter(|path| seen.insert(std::fs::canonicalize(path).unwrap_or_else(|_| path.clone())))
        .collect()
}

fn binaries_on_path() -> Vec<PathBuf> {
    let name = if cfg!(windows) { "java.exe" } else { "java" };
    // On macOS /usr/bin/java only forwards to an install that is already listed.
    let macos_stub = Path::new("/usr/bin/java");
    env::var_os("PATH")
        .map(|path| {
            env::split_paths(&path)
                .map(|dir| dir.join(name))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
        .into_iter()
        .filter(|path| !(cfg!(target_os = "macos") && path == macos_stub))
        .collect()
}

/// Each subdirectory of `dir` as a Java home, unwrapping macOS bundles.
fn java_homes_in(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .flatten()
        .map(|entry| entry.path())
        .map(|path| {
            // macOS bundles keep the actual home in Contents/Home; Mojang runtimes add jre.bundle.
            ["Contents/Home", "jre.bundle/Contents/Home"]
                .into_iter()
                .map(|home| path.join(home))
                .find(|home| home.is_dir())
                .unwrap_or(path)
        })
        .collect()
}

#[cfg(target_os = "macos")]
fn platform_homes(home: Option<&Path>) -> Vec<PathBuf> {
    let mut homes = java_homes_in(Path::new("/Library/Java/JavaVirtualMachines"));
    homes.extend(java_homes_in(Path::new(
        "/System/Library/Java/JavaVirtualMachines",
    )));
    if let Some(home) = home {
        homes.extend(java_homes_in(
            &home.join("Library/Java/JavaVirtualMachines"),
        ));
    }
    for prefix in ["/opt/homebrew/opt", "/usr/local/opt"] {
        let Ok(entries) = std::fs::read_dir(prefix) else {
            continue;
        };
        homes.extend(
            entries
                .flatten()
                .filter(|entry| entry.file_name().to_string_lossy().starts_with("openjdk"))
                .map(|entry| entry.path().join("libexec/openjdk.jdk/Contents/Home")),
        );
    }
    homes
}

#[cfg(target_os = "windows")]
fn platform_homes(_home: Option<&Path>) -> Vec<PathBuf> {
    let vendors = [
        "Java",
        "Eclipse Adoptium",
        "Eclipse Foundation",
        "Microsoft",
        "Zulu",
        "BellSoft",
        "Amazon Corretto",
    ];
    ["ProgramFiles", "ProgramFiles(x86)", "ProgramW6432"]
        .into_iter()
        .filter_map(env::var_os)
        .flat_map(|root| {
            let root = PathBuf::from(root);
            vendors
                .iter()
                .flat_map(move |vendor| java_homes_in(&root.join(vendor)))
        })
        .collect()
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn platform_homes(_home: Option<&Path>) -> Vec<PathBuf> {
    [
        "/usr/lib/jvm",
        "/usr/lib64/jvm",
        "/usr/lib32/jvm",
        "/usr/java",
        "/opt/jdk",
        "/opt/jdks",
    ]
    .into_iter()
    .flat_map(|dir| java_homes_in(Path::new(dir)))
    .collect()
}
