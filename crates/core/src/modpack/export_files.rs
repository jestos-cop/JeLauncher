use std::path::Path;

use crate::resources::is_source_entry;

/// Folders whose files may be published on a platform and can be referenced instead of bundled.
const RESOLVABLE_DIRS: [&str; 3] = ["mods", "resourcepacks", "shaderpacks"];
pub(super) const OVERRIDES_DIR: &str = "overrides";

pub(super) fn is_resolvable(path: &Path) -> bool {
    let in_resolvable_dir = path
        .parent()
        .and_then(|parent| parent.to_str())
        .is_some_and(|parent| RESOLVABLE_DIRS.contains(&parent));
    let extension = path.extension().and_then(|extension| extension.to_str());
    in_resolvable_dir && matches!(extension, Some("jar" | "zip"))
}

/// Source bookkeeping of the launcher, which other launchers rebuild themselves.
pub(super) fn is_source_metadata(path: &Path) -> bool {
    path.components()
        .any(|component| component.as_os_str().to_str().is_some_and(is_source_entry))
}

/// Archive entry name with forward slashes on every platform.
pub(super) fn entry_name(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_top_level_archives_in_content_folders_are_resolvable() {
        assert!(is_resolvable(Path::new("mods/sodium.jar")));
        assert!(is_resolvable(Path::new("resourcepacks/faithful.zip")));
        assert!(!is_resolvable(Path::new("mods/sodium.jar.disabled")));
        assert!(!is_resolvable(Path::new("mods/nested/a.jar")));
        assert!(!is_resolvable(Path::new("config/a.jar")));
    }

    #[test]
    fn source_files_are_never_exported() {
        assert!(is_source_metadata(Path::new("mods/.index/sodium.pw.toml")));
        assert!(is_source_metadata(Path::new("shaderpacks/bsl.pw.toml")));
        assert!(!is_source_metadata(Path::new("mods/sodium.jar")));
    }
}
