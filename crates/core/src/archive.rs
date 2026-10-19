use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::{Error, Result};

/// Extracts the entries of a zip archive into `dest`. `map` turns an entry's (already
/// sanitized) relative path into a path below `dest`, or `None` to skip it.
pub fn extract(
    archive_path: &Path,
    dest: &Path,
    map: impl Fn(&Path) -> Option<PathBuf>,
) -> Result<()> {
    let mut archive = open(archive_path)?;
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

/// Names of every file entry in a zip archive.
pub fn entry_names(archive_path: &Path) -> Result<Vec<String>> {
    let archive = open(archive_path)?;
    Ok(archive
        .file_names()
        .flatten()
        .filter(|name| !name.ends_with('/'))
        .map(|name| name.into_owned())
        .collect())
}

pub fn read_entry_to_string(archive_path: &Path, name: &str) -> Result<String> {
    let mut archive = open(archive_path)?;
    let mut entry = archive
        .by_name(name)
        .map_err(|err| Error::archive(archive_path, err))?;
    let mut text = String::new();
    entry
        .read_to_string(&mut text)
        .map_err(|err| Error::io(archive_path, err))?;
    Ok(text)
}

fn open(archive_path: &Path) -> Result<zip::ZipArchive<File>> {
    let file = File::open(archive_path).map_err(|err| Error::io(archive_path, err))?;
    zip::ZipArchive::new(file).map_err(|err| Error::archive(archive_path, err))
}

/// Zips every file under `source` into `target`, storing entries under `prefix`.
pub fn zip_dir(source: &Path, target: &Path, prefix: &Path) -> Result<()> {
    let file = File::create(target).map_err(|err| Error::io(target, err))?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default();
    let mut pending = vec![source.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir)
            .map_err(|err| Error::io(&dir, err))?
            .flatten()
        {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            let Ok(relative) = path.strip_prefix(source) else {
                continue;
            };
            let name = prefix.join(relative).to_string_lossy().replace('\\', "/");
            zip.start_file(name, options)
                .map_err(|err| Error::archive(target, err))?;
            let mut input = File::open(&path).map_err(|err| Error::io(&path, err))?;
            std::io::copy(&mut input, &mut zip).map_err(|err| Error::io(&path, err))?;
        }
    }
    zip.finish().map_err(|err| Error::archive(target, err))?;
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
