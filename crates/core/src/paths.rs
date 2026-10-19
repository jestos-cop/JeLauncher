use std::path::{Path, PathBuf};

use directories::ProjectDirs;

use crate::{Error, Result};

pub const DATA_DIR_ENV: &str = "JELAUNCHER_DATA_DIR";
/// A file with this name next to the executable keeps all data in the executable's folder.
pub const PORTABLE_MARKER: &str = "portable.txt";

#[derive(Debug, Clone)]
pub struct DataPaths {
    root: PathBuf,
}

impl DataPaths {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Picks the data folder: `JELAUNCHER_DATA_DIR` if set, the executable's folder in
    /// portable mode, otherwise the platform's application data folder.
    pub fn detect() -> Result<Self> {
        if let Some(dir) = std::env::var_os(DATA_DIR_ENV).filter(|dir| !dir.is_empty()) {
            return Ok(Self::new(dir));
        }
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(Path::to_path_buf));
        match exe_dir {
            Some(dir) if is_portable(&dir) => Ok(Self::new(dir)),
            _ => Self::platform_default(),
        }
    }

    pub fn platform_default() -> Result<Self> {
        let dirs = ProjectDirs::from("", "", "JeLauncher").ok_or(Error::NoDataDir)?;
        Ok(Self::new(dirs.data_dir()))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn accounts_file(&self) -> PathBuf {
        self.root.join("accounts.json")
    }

    pub fn settings_file(&self) -> PathBuf {
        self.root.join("settings.json")
    }

    pub fn instances(&self) -> PathBuf {
        self.root.join("instances")
    }

    /// Scratch space for downloads that are unpacked elsewhere afterwards.
    pub fn downloads_cache(&self) -> PathBuf {
        self.root.join("cache").join("downloads")
    }

    pub fn logs(&self) -> PathBuf {
        self.root.join("logs")
    }

    pub fn icons(&self) -> PathBuf {
        self.root.join("icons")
    }

    pub fn java(&self) -> PathBuf {
        self.root.join("java")
    }

    pub fn meta(&self) -> PathBuf {
        self.root.join("meta")
    }

    pub fn libraries(&self) -> PathBuf {
        self.root.join("libraries")
    }

    pub fn assets(&self) -> PathBuf {
        self.root.join("assets")
    }

    pub fn versions(&self) -> PathBuf {
        self.root.join("versions")
    }
}

fn is_portable(exe_dir: &Path) -> bool {
    exe_dir.join(PORTABLE_MARKER).is_file()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portable_marker_switches_the_data_folder() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|err| Error::io("tempdir", err))?;
        assert!(!is_portable(dir.path()));
        let marker = dir.path().join(PORTABLE_MARKER);
        std::fs::write(&marker, "").map_err(|err| Error::io(&marker, err))?;
        assert!(is_portable(dir.path()));
        Ok(())
    }
}
