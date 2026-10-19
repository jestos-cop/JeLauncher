use std::path::Path;

use super::Instance;
use crate::{Error, Result};

pub fn rename_instance(instances_dir: &Path, id: &str, name: &str) -> Result<()> {
    let name = name.trim();
    if name.is_empty() {
        return Err(Error::InvalidInput(
            "instance name must not be empty".into(),
        ));
    }
    let mut instance = Instance::open(instances_dir, id)?;
    instance.config.name = name.to_owned();
    instance.save()
}

pub fn set_instance_group(instances_dir: &Path, id: &str, group: Option<String>) -> Result<()> {
    let mut instance = Instance::open(instances_dir, id)?;
    instance.config.group = group.map(|g| g.trim().to_owned()).filter(|g| !g.is_empty());
    instance.save()
}

pub fn delete_instance(instances_dir: &Path, id: &str) -> Result<()> {
    let instance = Instance::open(instances_dir, id)?;
    std::fs::remove_dir_all(&instance.root).map_err(|err| Error::io(&instance.root, err))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instance::{NewInstance, create_instance};

    #[test]
    fn rename_group_and_delete() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|err| Error::io("tempdir", err))?;
        let instance = create_instance(
            dir.path(),
            NewInstance {
                name: "Old".into(),
                minecraft_version: "1.21.4".into(),
                loader: None,
                group: None,
                icon_key: None,
            },
        )?;

        rename_instance(dir.path(), &instance.id, "  New  ")?;
        set_instance_group(dir.path(), &instance.id, Some("Modded".into()))?;
        let reopened = Instance::open(dir.path(), &instance.id)?;
        assert_eq!(reopened.config.name, "New");
        assert_eq!(reopened.config.group.as_deref(), Some("Modded"));

        set_instance_group(dir.path(), &instance.id, Some("  ".into()))?;
        assert_eq!(Instance::open(dir.path(), &instance.id)?.config.group, None);

        delete_instance(dir.path(), &instance.id)?;
        assert!(!instance.root.exists());
        Ok(())
    }
}
