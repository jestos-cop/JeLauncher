mod metadata;

use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::Serialize;

pub use metadata::{ModMetadata, read_mod_metadata};

use crate::{Error, Result};

const DISABLED_SUFFIX: &str = ".disabled";

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ModFile {
    pub file_name: String,
    pub enabled: bool,
    pub size: u64,
    /// Unix time in milliseconds.
    pub modified: i64,
    pub metadata: Option<ModMetadata>,
}

/// Lists jar and zip mods in `mods_dir`, including disabled ones.
pub fn list_mods(mods_dir: &Path) -> Result<Vec<ModFile>> {
    let entries = match std::fs::read_dir(mods_dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(Error::io(mods_dir, err)),
    };
    let mut mods: Vec<ModFile> = entries
        .flatten()
        .filter_map(|entry| mod_file(&entry.path()))
        .collect();
    mods.sort_by_key(|m| display_name(m).to_lowercase());
    Ok(mods)
}

fn mod_file(path: &Path) -> Option<ModFile> {
    let file_name = path.file_name()?.to_str()?.to_owned();
    let base = file_name
        .strip_suffix(DISABLED_SUFFIX)
        .unwrap_or(&file_name);
    if !(base.ends_with(".jar") || base.ends_with(".zip")) {
        return None;
    }
    let metadata = std::fs::metadata(path).ok()?;
    let modified = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX));
    Some(ModFile {
        enabled: !file_name.ends_with(DISABLED_SUFFIX),
        size: metadata.len(),
        modified,
        metadata: read_mod_metadata(path).ok().flatten(),
        file_name,
    })
}

fn display_name(file: &ModFile) -> &str {
    file.metadata.as_ref().map_or(&file.file_name, |m| &m.name)
}

pub fn set_mod_enabled(mods_dir: &Path, file_name: &str, enabled: bool) -> Result<String> {
    let current = mod_path(mods_dir, file_name)?;
    let base = file_name.strip_suffix(DISABLED_SUFFIX).unwrap_or(file_name);
    let target_name = if enabled {
        base.to_owned()
    } else {
        format!("{base}{DISABLED_SUFFIX}")
    };
    let target = mods_dir.join(&target_name);
    if current != target {
        std::fs::rename(&current, &target).map_err(|err| Error::io(&current, err))?;
    }
    Ok(target_name)
}

pub fn remove_mod(mods_dir: &Path, file_name: &str) -> Result<()> {
    let path = mod_path(mods_dir, file_name)?;
    std::fs::remove_file(&path).map_err(|err| Error::io(&path, err))
}

/// Resolves a file name inside `mods_dir`, rejecting anything that could escape it.
fn mod_path(mods_dir: &Path, file_name: &str) -> Result<PathBuf> {
    let is_plain_name =
        Path::new(file_name).file_name().and_then(|n| n.to_str()) == Some(file_name);
    if !is_plain_name {
        return Err(Error::InvalidInput(format!(
            "invalid mod file name: {file_name}"
        )));
    }
    Ok(mods_dir.join(file_name))
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    fn write_jar(path: &Path, fabric_json: &str) -> Result<()> {
        let file = std::fs::File::create(path).map_err(|err| Error::io(path, err))?;
        let mut zip = zip::ZipWriter::new(file);
        zip.start_file("fabric.mod.json", zip::write::SimpleFileOptions::default())
            .map_err(|err| Error::archive(path, err))?;
        zip.write_all(fabric_json.as_bytes())
            .map_err(|err| Error::io(path, err))?;
        zip.finish().map_err(|err| Error::archive(path, err))?;
        Ok(())
    }

    #[test]
    fn lists_toggles_and_removes_mods() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|err| Error::io("tempdir", err))?;
        write_jar(
            &dir.path().join("sodium.jar"),
            r#"{"id":"sodium","name":"Sodium","version":"0.6"}"#,
        )?;
        std::fs::write(dir.path().join("notes.txt"), "x").map_err(|err| Error::io("notes", err))?;

        let mods = list_mods(dir.path())?;
        assert_eq!(mods.len(), 1);
        assert_eq!(
            mods[0].metadata.as_ref().map(|m| m.name.as_str()),
            Some("Sodium")
        );
        assert!(mods[0].enabled);

        let disabled = set_mod_enabled(dir.path(), "sodium.jar", false)?;
        assert_eq!(disabled, "sodium.jar.disabled");
        assert!(!list_mods(dir.path())?[0].enabled);

        assert!(remove_mod(dir.path(), "../escape.jar").is_err());
        remove_mod(dir.path(), &disabled)?;
        assert!(list_mods(dir.path())?.is_empty());
        Ok(())
    }
}
