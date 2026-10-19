use std::path::{Path, PathBuf};

use directories::ProjectDirs;

use crate::{Error, Result};

#[derive(Debug, Clone)]
pub struct DataPaths {
    root: PathBuf,
}

impl DataPaths {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
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
