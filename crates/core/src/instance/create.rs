use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::{INSTANCE_CONFIG_FILE, Instance, InstanceConfig};
use crate::minecraft::LoaderSpec;
use crate::{Error, Result};

#[derive(Debug, Clone, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct NewInstance {
    pub name: String,
    pub minecraft_version: String,
    pub loader: Option<LoaderSpec>,
    pub group: Option<String>,
    pub icon_key: Option<String>,
}

/// Creates the instance directory and config. Game files are fetched on first launch.
pub fn create_instance(instances_dir: &Path, new: NewInstance) -> Result<Instance> {
    let name = new.name.trim();
    if name.is_empty() {
        return Err(Error::InvalidInput(
            "instance name must not be empty".into(),
        ));
    }
    let id = unique_id(instances_dir, name);
    let root = instances_dir.join(&id);
    let instance = Instance {
        id,
        root: root.clone(),
        config: InstanceConfig {
            name: name.to_owned(),
            minecraft_version: new.minecraft_version,
            loader: new.loader,
            group: new.group.filter(|group| !group.trim().is_empty()),
            icon_key: new.icon_key.unwrap_or_else(|| "default".into()),
            ..Default::default()
        },
    };
    fs::create_dir_all(instance.game_dir()).map_err(|err| Error::io(instance.game_dir(), err))?;
    instance.config.save(&root.join(INSTANCE_CONFIG_FILE))?;
    Ok(instance)
}

/// Removes a freshly created instance when dropped before [`keep`](Self::keep), so failed or
/// cancelled installs leave nothing behind.
pub(crate) struct DraftGuard(Option<PathBuf>);

impl DraftGuard {
    pub(crate) fn new(instance: &Instance) -> Self {
        Self(Some(instance.root.clone()))
    }

    pub(crate) fn keep(mut self) {
        self.0 = None;
    }
}

impl Drop for DraftGuard {
    fn drop(&mut self) {
        if let Some(root) = self.0.take() {
            let _ = fs::remove_dir_all(root);
        }
    }
}

/// Directory-safe id derived from the name, with a numeric suffix on collision.
fn unique_id(instances_dir: &Path, name: &str) -> String {
    let base: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let base = base.trim_matches('.').to_owned();
    let base = if base.is_empty() {
        "instance".to_owned()
    } else {
        base
    };

    std::iter::once(base.clone())
        .chain((1..).map(|n| format!("{base}_{n}")))
        .find(|candidate| !instances_dir.join(candidate).exists())
        .unwrap_or(base)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_instance(name: &str) -> NewInstance {
        NewInstance {
            name: name.into(),
            minecraft_version: "1.21.4".into(),
            loader: None,
            group: None,
            icon_key: None,
        }
    }

    #[test]
    fn creates_unique_directories() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|err| Error::io("tempdir", err))?;
        let first = create_instance(dir.path(), new_instance("My Pack: 1/2"))?;
        let second = create_instance(dir.path(), new_instance("My Pack: 1/2"))?;
        assert_eq!(first.id, "My_Pack__1_2");
        assert_eq!(second.id, "My_Pack__1_2_1");
        assert!(first.game_dir().is_dir());

        let reopened = Instance::open(dir.path(), &first.id)?;
        assert_eq!(reopened.config.minecraft_version, "1.21.4");
        Ok(())
    }

    #[test]
    fn rejects_blank_names() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|err| Error::io("tempdir", err))?;
        assert!(create_instance(dir.path(), new_instance("   ")).is_err());
        Ok(())
    }

    #[test]
    fn draft_guard_removes_instances_that_are_not_kept() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|err| Error::io("tempdir", err))?;
        let dropped = create_instance(dir.path(), new_instance("Dropped"))?;
        drop(DraftGuard::new(&dropped));
        let kept = create_instance(dir.path(), new_instance("Kept"))?;
        DraftGuard::new(&kept).keep();
        assert!(!dropped.root.exists());
        assert!(kept.root.exists());
        Ok(())
    }
}
