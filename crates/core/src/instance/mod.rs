mod config;
mod copy;
mod create;
mod export;
mod list;
mod manage;

use std::path::{Path, PathBuf};

pub use config::InstanceConfig;
pub use copy::{CopyOptions, copy_instance};
pub use create::{NewInstance, create_instance};
pub use export::{ExportEntry, export_instance};
pub use list::{InstanceSummary, discover_instances};
pub use manage::{
    delete_instance, rename_instance, set_instance_group, set_instance_notes, set_instance_settings,
};

pub const INSTANCE_CONFIG_FILE: &str = "instance.json";
pub const GAME_DIR: &str = "minecraft";

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

    pub fn resource_dir(&self, kind: crate::resources::ResourceKind) -> PathBuf {
        self.game_dir().join(kind.install_dir())
    }

    pub fn saves_dir(&self) -> PathBuf {
        self.game_dir().join("saves")
    }

    pub fn servers_file(&self) -> PathBuf {
        self.game_dir().join("servers.dat")
    }

    pub fn screenshots_dir(&self) -> PathBuf {
        self.game_dir().join("screenshots")
    }

    pub fn backups_dir(&self) -> PathBuf {
        self.root.join("backups")
    }

    pub fn natives_dir(&self) -> PathBuf {
        self.root.join("natives")
    }

    pub fn save(&self) -> crate::Result<()> {
        self.config.save(&self.root.join(INSTANCE_CONFIG_FILE))
    }
}
