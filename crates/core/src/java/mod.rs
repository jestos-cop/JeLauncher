mod check;
mod detect;
mod runtime;

pub use check::{JavaInstall, check_java, parse_major_version};
pub use detect::{candidate_paths, detect_java_installs};
pub use runtime::{LEGACY_COMPONENT, install_runtime, runtime_java};

/// Games that ask for Java 8 often use LaunchWrapper or old mods that break on Java 9
/// and later, so they only get Java 8 itself.
const LEGACY_JAVA_MAJOR: u32 = 8;

/// Picks the install that best fits `required_major`: an exact major match first,
/// then the closest newer one, preferring the host architecture.
pub fn pick_java(installs: &[JavaInstall], required_major: u32) -> Option<&JavaInstall> {
    let native = |install: &&JavaInstall| install.is_native_arch();
    let accepts_newer = required_major > LEGACY_JAVA_MAJOR;
    let exact = installs.iter().filter(|i| i.major == required_major);
    let newer = || {
        let mut candidates: Vec<_> = installs
            .iter()
            .filter(|i| accepts_newer && i.major > required_major)
            .collect();
        candidates.sort_by_key(|i| i.major);
        candidates.into_iter()
    };
    exact
        .clone()
        .find(native)
        .or_else(|| exact.clone().next())
        .or_else(|| newer().find(native))
        .or_else(|| newer().next())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn install(major: u32, arch: &str) -> JavaInstall {
        JavaInstall {
            path: format!("/java{major}-{arch}"),
            version: major.to_string(),
            major,
            arch: arch.into(),
            vendor: String::new(),
        }
    }

    #[test]
    fn prefers_exact_then_closest_newer() {
        let host = std::env::consts::ARCH;
        let installs = [install(25, host), install(21, host), install(17, host)];
        assert_eq!(pick_java(&installs, 21).map(|i| i.major), Some(21));
        assert_eq!(pick_java(&installs, 18).map(|i| i.major), Some(21));
        assert_eq!(pick_java(&installs, 26).map(|i| i.major), None);
    }

    #[test]
    fn java_8_games_only_run_on_java_8() {
        let host = std::env::consts::ARCH;
        let installs = [install(21, host), install(17, host)];
        assert_eq!(pick_java(&installs, 8).map(|i| i.major), None);
        let with_8 = [install(21, host), install(8, host)];
        assert_eq!(pick_java(&with_8, 8).map(|i| i.major), Some(8));
    }

    #[test]
    fn prefers_native_architecture() {
        let host = std::env::consts::ARCH;
        let other = if host == "aarch64" {
            "x86_64"
        } else {
            "aarch64"
        };
        let installs = [install(21, other), install(21, host)];
        assert_eq!(
            pick_java(&installs, 21).map(|i| i.arch.as_str()),
            Some(host)
        );
    }
}
