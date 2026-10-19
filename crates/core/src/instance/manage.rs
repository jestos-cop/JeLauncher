use std::path::Path;

use super::Instance;
use crate::minecraft::LoaderSpec;
use crate::settings::InstanceSettings;
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

/// Switches the game version and mod loader; game files are fetched on the next launch.
pub fn set_instance_version(
    instances_dir: &Path,
    id: &str,
    minecraft_version: &str,
    loader: Option<LoaderSpec>,
) -> Result<()> {
    let minecraft_version = minecraft_version.trim();
    if minecraft_version.is_empty() {
        return Err(Error::InvalidInput(
            "a Minecraft version is required".into(),
        ));
    }
    if loader
        .as_ref()
        .is_some_and(|loader| loader.version.trim().is_empty())
    {
        return Err(Error::InvalidInput("choose a loader version".into()));
    }
    let mut instance = Instance::open(instances_dir, id)?;
    instance.config.minecraft_version = minecraft_version.to_owned();
    instance.config.loader = loader;
    instance.save()
}

pub fn set_instance_notes(instances_dir: &Path, id: &str, notes: String) -> Result<()> {
    let mut instance = Instance::open(instances_dir, id)?;
    instance.config.notes = notes;
    instance.save()
}

pub fn set_instance_settings(
    instances_dir: &Path,
    id: &str,
    settings: InstanceSettings,
) -> Result<()> {
    let mut instance = Instance::open(instances_dir, id)?;
    instance.config.settings = settings;
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
    fn rename_group_notes_and_delete() -> Result<()> {
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
        set_instance_notes(dir.path(), &instance.id, "Seed: 42\n".into())?;
        let fabric = LoaderSpec {
            kind: crate::minecraft::LoaderKind::Fabric,
            version: "0.19.5".into(),
        };
        set_instance_version(dir.path(), &instance.id, "1.21.5", Some(fabric.clone()))?;
        assert!(set_instance_version(dir.path(), &instance.id, " ", None).is_err());
        let reopened = Instance::open(dir.path(), &instance.id)?;
        assert_eq!(reopened.config.name, "New");
        assert_eq!(reopened.config.group.as_deref(), Some("Modded"));
        assert_eq!(reopened.config.notes, "Seed: 42\n");
        assert_eq!(reopened.config.minecraft_version, "1.21.5");
        assert_eq!(reopened.config.loader, Some(fabric));

        set_instance_group(dir.path(), &instance.id, Some("  ".into()))?;
        assert_eq!(Instance::open(dir.path(), &instance.id)?.config.group, None);

        delete_instance(dir.path(), &instance.id)?;
        assert!(!instance.root.exists());
        Ok(())
    }
}
