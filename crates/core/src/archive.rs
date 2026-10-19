use std::fs::File;
use std::path::{Path, PathBuf};

use crate::{Error, Result};

/// Extracts the entries of a zip archive into `dest`. `map` turns an entry's (already
/// sanitized) relative path into a path below `dest`, or `None` to skip it.
pub fn extract(
    archive_path: &Path,
    dest: &Path,
    map: impl Fn(&Path) -> Option<PathBuf>,
) -> Result<()> {
    let file = File::open(archive_path).map_err(|err| Error::io(archive_path, err))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|err| Error::archive(archive_path, err))?;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|err| Error::archive(archive_path, err))?;
        if entry.is_dir() {
            continue;
        }
        let Some(relative) = entry.enclosed_name().and_then(|name| map(&name)) else {
            continue;
        };
        let target = dest.join(relative);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|err| Error::io(parent, err))?;
        }
        let mut out = File::create(&target).map_err(|err| Error::io(&target, err))?;
        std::io::copy(&mut entry, &mut out).map_err(|err| Error::io(&target, err))?;
    }
    Ok(())
}

/// Accepts a relative path only if it cannot escape the directory it is joined to.
pub fn safe_relative_path(path: &str) -> Option<PathBuf> {
    let path = Path::new(path);
    let is_safe = !path.as_os_str().is_empty()
        && path
            .components()
            .all(|component| matches!(component, std::path::Component::Normal(_)));
    is_safe.then(|| path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_escaping_paths() {
        assert_eq!(
            safe_relative_path("mods/a.jar"),
            Some(PathBuf::from("mods/a.jar"))
        );
        assert_eq!(safe_relative_path("../evil.jar"), None);
        assert_eq!(safe_relative_path("/etc/passwd"), None);
        assert_eq!(safe_relative_path("mods/../../x"), None);
        assert_eq!(safe_relative_path(""), None);
    }
}
