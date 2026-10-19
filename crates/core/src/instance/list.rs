use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use serde::Serialize;

use super::{INSTANCE_CONFIG_FILE, InstanceConfig};
use crate::{Error, Result};

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct InstanceSummary {
    pub id: String,
    pub name: String,
    pub icon_key: String,
    pub last_launch_time: i64,
    pub total_time_played: i64,
}

pub fn discover_instances(root: &Path) -> Result<Vec<InstanceSummary>> {
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(Error::io(root, err)),
    };

    let mut instances = Vec::new();
    for entry in entries {
        let dir = entry.map_err(|err| Error::io(root, err))?.path();
        let config_path = dir.join(INSTANCE_CONFIG_FILE);
        if !config_path.is_file() {
            continue;
        }
        let Some(id) = dir.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        // Instances with an unreadable config are still listed, using default values.
        let config = InstanceConfig::load(&config_path).unwrap_or_default();
        instances.push(InstanceSummary {
            id: id.to_owned(),
            name: config.name,
            icon_key: config.icon_key,
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
        let write = |id: &str, config: Option<&str>| -> Result<()> {
            let path = dir.path().join(id);
            fs::create_dir(&path).map_err(|err| Error::io(&path, err))?;
            if let Some(config) = config {
                let file = path.join(INSTANCE_CONFIG_FILE);
                fs::write(&file, config).map_err(|err| Error::io(&file, err))?;
            }
            Ok(())
        };
        write("b", Some("[General]\nname=Beta\n"))?;
        write("a", Some("[General]\nname=alpha\n"))?;
        write("not-an-instance", None)?;

        let names: Vec<_> = discover_instances(dir.path())?
            .into_iter()
            .map(|instance| instance.name)
            .collect();
        assert_eq!(names, ["alpha", "Beta"]);
        Ok(())
    }
}
