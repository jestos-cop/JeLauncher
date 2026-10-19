use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::{Error, Result};

/// Joins a single file or folder name to `dir`, rejecting anything that could escape it.
pub fn child_path(dir: &Path, name: &str) -> Result<PathBuf> {
    let is_plain_name = Path::new(name).file_name().and_then(|n| n.to_str()) == Some(name);
    if !is_plain_name {
        return Err(Error::InvalidInput(format!("invalid name: {name}")));
    }
    Ok(dir.join(name))
}

/// Reads a JSON file, or returns `None` if it does not exist.
pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<Option<T>> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(Error::io(path, err)),
    };
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|err| Error::json(path, err))
}

/// Writes through a temporary file so a crash never leaves a half-written file behind.
pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|err| Error::json(path, err))?;
    write_atomic(path, &bytes)
}

pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| Error::io(parent, err))?;
    }
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, bytes).map_err(|err| Error::io(&tmp, err))?;
    fs::rename(&tmp, path).map_err(|err| Error::io(path, err))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_round_trip() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|err| Error::io("tempdir", err))?;
        let path = dir.path().join("nested/value.json");
        assert_eq!(read_json::<Vec<u32>>(&path)?, None);
        write_json(&path, &vec![1, 2, 3])?;
        assert_eq!(read_json::<Vec<u32>>(&path)?, Some(vec![1, 2, 3]));
        Ok(())
    }
}
