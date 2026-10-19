mod config;
mod create;
mod list;

use std::path::{Path, PathBuf};

pub use config::InstanceConfig;
pub use create::{NewInstance, create_instance};
pub use list::{InstanceSummary, discover_instances};

pub const INSTANCE_CONFIG_FILE: &str = "instance.json";
const GAME_DIR: &str = "minecraft";

/// An instance directory on disk together with its parsed config.
#[derive(Debug, Clone)]
pub struct Instance {
    pub id: String,
    pub root: PathBuf,
    pub config: InstanceConfig,
}

impl Instance {
    pub fn open(instances_dir: &Path, id: &str) -> crate::Result<Self> {
        let root = instances_dir.join(id);
        let config = InstanceConfig::load(&root.join(INSTANCE_CONFIG_FILE))?
            .ok_or_else(|| crate::Error::UnknownInstance(id.to_owned()))?;
        Ok(Self {
            id: id.to_owned(),
            root,
            config,
        })
    }

    pub fn game_dir(&self) -> PathBuf {
        self.root.join(GAME_DIR)
    }

    pub fn natives_dir(&self) -> PathBuf {
        self.root.join("natives")
    }

    pub fn save(&self) -> crate::Result<()> {
        self.config.save(&self.root.join(INSTANCE_CONFIG_FILE))
    }
}
