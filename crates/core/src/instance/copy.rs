use std::path::Path;

use serde::Deserialize;

use super::{Instance, InstanceConfig, NewInstance, create_instance};
use crate::archive::walk_files;
use crate::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct CopyOptions {
    pub name: String,
    pub group: Option<String>,
    pub copy_saves: bool,
    pub copy_game_options: bool,
    pub copy_resource_packs: bool,
    pub copy_shader_packs: bool,
    pub copy_servers: bool,
    pub copy_mods: bool,
    pub copy_screenshots: bool,
    pub keep_play_time: bool,
    /// Hard-links game files instead of copying them; worlds are always copied.
    pub use_hard_links: bool,
}

impl CopyOptions {
    /// Top-level game directory entries left out of the copy.
    fn excluded(&self) -> Vec<&'static str> {
        let groups: [(bool, &[&'static str]); 7] = [
            (self.copy_saves, &["saves"]),
            (self.copy_game_options, &["options.txt"]),
            (self.copy_resource_packs, &["resourcepacks", "texturepacks"]),
            (self.copy_shader_packs, &["shaderpacks"]),
            (
                self.copy_servers,
                &["servers.dat", "servers.dat_old", "server-resource-packs"],
            ),
            (self.copy_mods, &["coremods", "mods", "config"]),
            (self.copy_screenshots, &["screenshots"]),
        ];
        groups
            .into_iter()
            .filter(|(copy, _)| !copy)
            .flat_map(|(_, entries)| entries.iter().copied())
            .collect()
    }
}

/// Creates a new instance from `source` with the selected parts of its game directory.
/// The half-created copy is removed again if anything fails.
pub fn copy_instance(
    instances_dir: &Path,
    source: &Instance,
    options: &CopyOptions,
) -> Result<Instance> {
    let mut copy = create_instance(
        instances_dir,
        NewInstance {
            name: options.name.clone(),
            minecraft_version: source.config.minecraft_version.clone(),
            loader: source.config.loader.clone(),
            group: options.group.clone(),
            icon_key: Some(source.config.icon_key.clone()),
        },
    )?;
    if let Err(err) = copy_game_files(source, &copy, options) {
        let _ = std::fs::remove_dir_all(&copy.root);
        return Err(err);
    }
    copy.config = InstanceConfig {
        name: copy.config.name,
        group: copy.config.group,
        last_launch_time: 0,
        total_time_played: if options.keep_play_time {
            source.config.total_time_played
        } else {
            0
        },
        ..source.config.clone()
    };
    copy.save()?;
    Ok(copy)
}

fn copy_game_files(source: &Instance, copy: &Instance, options: &CopyOptions) -> Result<()> {
    let from = source.game_dir();
    if !from.is_dir() {
        return Ok(());
    }
    let to = copy.game_dir();
    let excluded = options.excluded();
    for relative in walk_files(&from)? {
        if excluded.iter().any(|entry| relative.starts_with(entry)) {
            continue;
        }
        let target = to.join(&relative);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|err| Error::io(parent, err))?;
        }
        let file = from.join(&relative);
        let linked = options.use_hard_links
            && !relative.starts_with("saves")
            && std::fs::hard_link(&file, &target).is_ok();
        if !linked {
            std::fs::copy(&file, &target).map_err(|err| Error::io(&file, err))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options(name: &str) -> CopyOptions {
        CopyOptions {
            name: name.into(),
            group: Some("Copies".into()),
            copy_saves: true,
            copy_game_options: true,
            copy_resource_packs: true,
            copy_shader_packs: true,
            copy_servers: true,
            copy_mods: true,
            copy_screenshots: true,
            keep_play_time: false,
            use_hard_links: false,
        }
    }

    #[test]
    fn copies_selected_contents_and_config() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|err| Error::io("tempdir", err))?;
        let mut source = create_instance(
            dir.path(),
            NewInstance {
                name: "Original".into(),
                minecraft_version: "1.21.4".into(),
                loader: None,
                group: None,
                icon_key: None,
            },
        )?;
        source.config.notes = "notes".into();
        source.config.total_time_played = 600;
        source.save()?;
        for file in [
            "options.txt",
            "saves/w/level.dat",
            "mods/a.jar",
            "config/a.toml",
        ] {
            let path = source.game_dir().join(file);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|err| Error::io(parent, err))?;
            }
            std::fs::write(&path, file).map_err(|err| Error::io(&path, err))?;
        }

        let copy = copy_instance(
            dir.path(),
            &source,
            &CopyOptions {
                copy_saves: false,
                use_hard_links: true,
                ..options("Copy")
            },
        )?;
        let copy = Instance::open(dir.path(), &copy.id)?;
        assert_eq!(copy.config.name, "Copy");
        assert_eq!(copy.config.group.as_deref(), Some("Copies"));
        assert_eq!(copy.config.notes, "notes");
        assert_eq!(copy.config.total_time_played, 0);
        assert!(copy.game_dir().join("mods/a.jar").is_file());
        assert!(copy.game_dir().join("options.txt").is_file());
        assert!(!copy.game_dir().join("saves").exists());

        let without_mods = copy_instance(
            dir.path(),
            &source,
            &CopyOptions {
                copy_mods: false,
                keep_play_time: true,
                ..options("Vanilla copy")
            },
        )?;
        assert!(!without_mods.game_dir().join("mods").exists());
        assert!(!without_mods.game_dir().join("config").exists());
        assert!(without_mods.game_dir().join("saves/w/level.dat").is_file());
        assert_eq!(without_mods.config.total_time_played, 600);
        Ok(())
    }
}
