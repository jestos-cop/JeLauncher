use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct World {
    /// Folder name inside `saves`.
    pub folder: String,
    pub name: String,
    pub game_mode: GameMode,
    pub hardcore: bool,
    /// Unix time in milliseconds.
    pub last_played: i64,
    pub seed: Option<String>,
    pub game_version: Option<String>,
    /// The world's `icon.png`, when the game has saved one.
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum GameMode {
    Survival,
    Creative,
    Adventure,
    Spectator,
    Unknown,
}

#[derive(Deserialize)]
struct LevelDat {
    #[serde(rename = "Data")]
    data: LevelData,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct LevelData {
    level_name: Option<String>,
    last_played: Option<i64>,
    game_type: Option<i32>,
    hardcore: Option<i8>,
    random_seed: Option<i64>,
    world_gen_settings: Option<WorldGenSettings>,
    version: Option<VersionInfo>,
}

#[derive(Deserialize)]
struct WorldGenSettings {
    seed: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct VersionInfo {
    name: Option<String>,
}

/// Lists worlds in a `saves` directory, newest first.
pub fn list_worlds(saves_dir: &Path) -> Result<Vec<World>> {
    let entries = match std::fs::read_dir(saves_dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(Error::io(saves_dir, err)),
    };
    let mut worlds: Vec<World> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.join("level.dat").is_file())
        .filter_map(|path| read_world(&path).ok())
        .collect();
    worlds.sort_by(|a, b| b.last_played.cmp(&a.last_played));
    Ok(worlds)
}

pub fn read_world(dir: &Path) -> Result<World> {
    let level_path = dir.join("level.dat");
    let file = File::open(&level_path).map_err(|err| Error::io(&level_path, err))?;
    let mut bytes = Vec::new();
    flate2::read::GzDecoder::new(file)
        .read_to_end(&mut bytes)
        .map_err(|err| Error::io(&level_path, err))?;
    let level: LevelDat = fastnbt::from_bytes(&bytes).map_err(|err| {
        Error::InvalidInput(format!("unreadable {}: {err}", level_path.display()))
    })?;
    let data = level.data;
    let folder = dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_owned();
    let seed = data
        .world_gen_settings
        .and_then(|settings| settings.seed)
        .or(data.random_seed);
    Ok(World {
        name: data.level_name.unwrap_or_else(|| folder.clone()),
        game_mode: match data.game_type {
            Some(0) => GameMode::Survival,
            Some(1) => GameMode::Creative,
            Some(2) => GameMode::Adventure,
            Some(3) => GameMode::Spectator,
            _ => GameMode::Unknown,
        },
        hardcore: data.hardcore.unwrap_or(0) != 0,
        last_played: data.last_played.unwrap_or(0),
        seed: seed.map(|seed| seed.to_string()),
        game_version: data.version.and_then(|version| version.name),
        icon: Some(dir.join("icon.png"))
            .filter(|icon| icon.is_file())
            .map(|icon| icon.to_string_lossy().into_owned()),
        folder,
    })
}

pub fn delete_world(saves_dir: &Path, folder: &str) -> Result<()> {
    let path = crate::fs::child_path(saves_dir, folder)?;
    std::fs::remove_dir_all(&path).map_err(|err| Error::io(&path, err))
}

/// Zips the world into `backups_dir` and returns the archive path.
pub fn backup_world(
    saves_dir: &Path,
    folder: &str,
    backups_dir: &Path,
    timestamp: &str,
) -> Result<PathBuf> {
    let source = crate::fs::child_path(saves_dir, folder)?;
    std::fs::create_dir_all(backups_dir).map_err(|err| Error::io(backups_dir, err))?;
    let target = backups_dir.join(format!("{folder}-{timestamp}.zip"));
    crate::archive::zip_dir(&source, &target, Path::new(folder))?;
    Ok(target)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    fn write_level_dat(dir: &Path, nbt: &impl Serialize) -> Result<()> {
        std::fs::create_dir_all(dir).map_err(|err| Error::io(dir, err))?;
        let bytes = fastnbt::to_bytes(nbt).map_err(|err| Error::InvalidInput(err.to_string()))?;
        let file = File::create(dir.join("level.dat")).map_err(|err| Error::io(dir, err))?;
        let mut gz = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        gz.write_all(&bytes).map_err(|err| Error::io(dir, err))?;
        gz.finish().map_err(|err| Error::io(dir, err))?;
        Ok(())
    }

    #[derive(Serialize)]
    struct Root {
        #[serde(rename = "Data")]
        data: Data,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "PascalCase")]
    struct Data {
        level_name: String,
        last_played: i64,
        game_type: i32,
        world_gen_settings: Gen,
    }

    #[derive(Serialize)]
    struct Gen {
        seed: i64,
    }

    #[test]
    fn reads_lists_backs_up_and_deletes_worlds() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|err| Error::io("tempdir", err))?;
        let saves = dir.path().join("saves");
        write_level_dat(
            &saves.join("world1"),
            &Root {
                data: Data {
                    level_name: "My World".into(),
                    last_played: 1_700_000_000_000,
                    game_type: 1,
                    world_gen_settings: Gen { seed: -42 },
                },
            },
        )?;
        std::fs::create_dir_all(saves.join("not-a-world")).map_err(|err| Error::io("dir", err))?;

        let worlds = list_worlds(&saves)?;
        assert_eq!(worlds.len(), 1);
        assert_eq!(worlds[0].name, "My World");
        assert_eq!(worlds[0].game_mode, GameMode::Creative);
        assert_eq!(worlds[0].seed.as_deref(), Some("-42"));

        let backup = backup_world(&saves, "world1", &dir.path().join("backups"), "test")?;
        assert!(backup.is_file());
        assert!(delete_world(&saves, "../saves").is_err());
        delete_world(&saves, "world1")?;
        assert!(list_worlds(&saves)?.is_empty());
        Ok(())
    }
}
