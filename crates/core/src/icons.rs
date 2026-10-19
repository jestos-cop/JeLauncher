use std::path::Path;

use serde::Serialize;

use crate::fs::child_path;
use crate::{Error, Result};

/// Icon keys with this prefix name a file in the user icons folder; other keys are built in.
pub const CUSTOM_PREFIX: &str = "custom:";
const IMAGE_EXTENSIONS: [&str; 5] = ["png", "jpg", "jpeg", "gif", "webp"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct CustomIcon {
    pub key: String,
    pub path: String,
}

pub fn list_custom_icons(dir: &Path) -> Result<Vec<CustomIcon>> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(Error::io(dir, err)),
    };
    let mut icons: Vec<_> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && is_image(path))
        .filter_map(|path| {
            let name = path.file_name()?.to_str()?.to_owned();
            Some(CustomIcon {
                key: format!("{CUSTOM_PREFIX}{name}"),
                path: path.to_string_lossy().into_owned(),
            })
        })
        .collect();
    icons.sort_by(|a, b| a.key.cmp(&b.key));
    Ok(icons)
}

/// Copies an image into the icons folder and returns its icon key.
pub fn add_custom_icon(dir: &Path, source: &Path) -> Result<String> {
    if !is_image(source) {
        return Err(Error::InvalidInput(format!(
            "{} is not a supported image",
            source.display()
        )));
    }
    let stem = source
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("icon");
    let extension = source
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("png")
        .to_ascii_lowercase();
    std::fs::create_dir_all(dir).map_err(|err| Error::io(dir, err))?;
    let name = std::iter::once(format!("{stem}.{extension}"))
        .chain((1..).map(|n| format!("{stem}_{n}.{extension}")))
        .find(|name| !dir.join(name).exists())
        .unwrap_or_else(|| format!("{stem}.{extension}"));
    let target = child_path(dir, &name)?;
    std::fs::copy(source, &target).map_err(|err| Error::io(source, err))?;
    Ok(format!("{CUSTOM_PREFIX}{name}"))
}

pub fn remove_custom_icon(dir: &Path, key: &str) -> Result<()> {
    let name = key
        .strip_prefix(CUSTOM_PREFIX)
        .ok_or_else(|| Error::InvalidInput(format!("{key} is a built-in icon")))?;
    let path = child_path(dir, name)?;
    std::fs::remove_file(&path).map_err(|err| Error::io(&path, err))
}

fn is_image(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            IMAGE_EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_lists_and_removes_custom_icons() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|err| Error::io("tempdir", err))?;
        let source = dir.path().join("Creeper.PNG");
        std::fs::write(&source, b"png").map_err(|err| Error::io(&source, err))?;
        let icons = dir.path().join("icons");

        assert_eq!(add_custom_icon(&icons, &source)?, "custom:Creeper.png");
        assert_eq!(add_custom_icon(&icons, &source)?, "custom:Creeper_1.png");
        let keys: Vec<_> = list_custom_icons(&icons)?
            .into_iter()
            .map(|icon| icon.key)
            .collect();
        assert_eq!(keys, ["custom:Creeper.png", "custom:Creeper_1.png"]);

        remove_custom_icon(&icons, "custom:Creeper.png")?;
        assert_eq!(list_custom_icons(&icons)?.len(), 1);
        assert!(remove_custom_icon(&icons, "grass").is_err());
        assert!(remove_custom_icon(&icons, "custom:../x.png").is_err());
        assert!(add_custom_icon(&icons, &dir.path().join("notes.txt")).is_err());
        Ok(())
    }
}
