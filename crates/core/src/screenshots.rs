use std::path::Path;
use std::time::UNIX_EPOCH;

use serde::Serialize;

use crate::fs::child_path;
use crate::{Error, Result};

const IMAGE_EXTENSIONS: [&str; 3] = ["png", "jpg", "jpeg"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Screenshot {
    pub file_name: String,
    pub path: String,
    pub size: u64,
    /// Unix time in milliseconds.
    pub modified: i64,
}

/// Lists the images in a `screenshots` directory, newest first.
pub fn list_screenshots(dir: &Path) -> Result<Vec<Screenshot>> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(Error::io(dir, err)),
    };
    let mut screenshots: Vec<_> = entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            let extension = path.extension()?.to_str()?.to_ascii_lowercase();
            if !IMAGE_EXTENSIONS.contains(&extension.as_str()) {
                return None;
            }
            let metadata = entry.metadata().ok().filter(std::fs::Metadata::is_file)?;
            let modified = metadata
                .modified()
                .ok()
                .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                .map_or(0, |elapsed| {
                    i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX)
                });
            Some(Screenshot {
                file_name: entry.file_name().into_string().ok()?,
                path: path.to_string_lossy().into_owned(),
                size: metadata.len(),
                modified,
            })
        })
        .collect();
    screenshots.sort_by(|a, b| {
        b.modified
            .cmp(&a.modified)
            .then_with(|| a.file_name.cmp(&b.file_name))
    });
    Ok(screenshots)
}

pub fn delete_screenshot(dir: &Path, file_name: &str) -> Result<()> {
    let path = child_path(dir, file_name)?;
    std::fs::remove_file(&path).map_err(|err| Error::io(&path, err))
}

/// Renames a screenshot, keeping its extension when the new name has none.
/// Returns the resulting file name.
pub fn rename_screenshot(dir: &Path, file_name: &str, new_name: &str) -> Result<String> {
    let source = child_path(dir, file_name)?;
    let new_name = new_name.trim();
    let new_name = match (Path::new(new_name).extension(), source.extension()) {
        (None, Some(extension)) => format!("{new_name}.{}", extension.to_string_lossy()),
        _ => new_name.to_owned(),
    };
    let target = child_path(dir, &new_name)?;
    if target.exists() {
        return Err(Error::InvalidInput(format!("{new_name} already exists")));
    }
    std::fs::rename(&source, &target).map_err(|err| Error::io(&source, err))?;
    Ok(new_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_renames_and_deletes_images() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|err| Error::io("tempdir", err))?;
        for name in ["2025-01-01_10.00.00.png", "notes.txt"] {
            std::fs::write(dir.path().join(name), b"x").map_err(|err| Error::io(name, err))?;
        }

        let names = |dir: &Path| -> Result<Vec<String>> {
            Ok(list_screenshots(dir)?
                .into_iter()
                .map(|shot| shot.file_name)
                .collect())
        };
        assert_eq!(names(dir.path())?, ["2025-01-01_10.00.00.png"]);

        let renamed = rename_screenshot(dir.path(), "2025-01-01_10.00.00.png", "base")?;
        assert_eq!(renamed, "base.png");
        assert!(rename_screenshot(dir.path(), "base.png", "../escape.png").is_err());
        std::fs::write(dir.path().join("other.png"), b"x")
            .map_err(|err| Error::io("other", err))?;
        assert!(rename_screenshot(dir.path(), "base.png", "other").is_err());

        delete_screenshot(dir.path(), "base.png")?;
        assert_eq!(names(dir.path())?, ["other.png"]);
        assert!(names(&dir.path().join("missing"))?.is_empty());
        Ok(())
    }
}
