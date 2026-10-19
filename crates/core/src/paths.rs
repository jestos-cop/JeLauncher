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

    pub fn instances(&self) -> PathBuf {
        self.root.join("instances")
    }
}
