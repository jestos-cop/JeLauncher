use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use serde::Serialize;

use super::{INSTANCE_CONFIG_FILE, InstanceConfig};
use crate::minecraft::LoaderSpec;
use crate::{Error, Result};

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct InstanceSummary {
    pub id: String,
    pub name: String,
    pub icon_key: String,
    pub group: Option<String>,
    pub minecraft_version: String,
    pub loader: Option<LoaderSpec>,
    pub last_launch_time: i64,
    pub total_time_played: i64,
}

/// Finds every directory under `root` that contains an instance config.
pub fn discover_instances(root: &Path) -> Result<Vec<InstanceSummary>> {
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(Error::io(root, err)),
    };

    let mut instances = Vec::new();
    for entry in entries {
        let dir = entry.map_err(|err| Error::io(root, err))?.path();
        let Some(id) = dir.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        // A broken config should not hide every other instance.
        let Ok(Some(config)) = InstanceConfig::load(&dir.join(INSTANCE_CONFIG_FILE)) else {
            continue;
        };
        instances.push(InstanceSummary {
            id: id.to_owned(),
            name: config.name,
            icon_key: config.icon_key,
            group: config.group,
            minecraft_version: config.minecraft_version,
            loader: config.loader,
            last_launch_time: config.last_launch_time,
            total_time_played: config.total_time_played,
        });
    }

    instances.sort_by_key(|instance| instance.name.to_lowercase());
    Ok(instances)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_root_yields_no_instances() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|err| Error::io("tempdir", err))?;
        assert!(discover_instances(&dir.path().join("absent"))?.is_empty());
        Ok(())
    }

    #[test]
    fn lists_only_directories_with_config() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|err| Error::io("tempdir", err))?;
        let write = |id: &str, name: Option<&str>| -> Result<()> {
            let path = dir.path().join(id);
            fs::create_dir(&path).map_err(|err| Error::io(&path, err))?;
            if let Some(name) = name {
                let config = InstanceConfig {
                    name: name.into(),
                    ..Default::default()
                };
                config.save(&path.join(INSTANCE_CONFIG_FILE))?;
            }
            Ok(())
        };
        write("b", Some("Beta"))?;
        write("a", Some("alpha"))?;
        write("not-an-instance", None)?;

        let names: Vec<_> = discover_instances(dir.path())?
            .into_iter()
            .map(|instance| instance.name)
            .collect();
        assert_eq!(names, ["alpha", "Beta"]);
        Ok(())
    }
}
