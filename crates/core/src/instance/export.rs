use std::path::{Path, PathBuf};

use serde::Serialize;

use super::{GAME_DIR, INSTANCE_CONFIG_FILE, Instance};
use crate::archive::{self, ZipBuilder};
use crate::{Error, Result};

/// Lists the paths, relative to the game directory, that the user chose not to export.
const PACK_IGNORE_FILE: &str = ".packignore";
const DEFAULT_BLOCKED: [&str; 5] = ["logs", "crash-reports", ".cache", ".fabric", ".quilt"];
const IGNORED_NAMES: [&str; 3] = [".DS_Store", "thumbs.db", "Thumbs.db"];

/// A top-level entry of the game directory as offered for export.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ExportEntry {
    pub path: String,
    pub is_dir: bool,
    pub blocked: bool,
}

impl Instance {
    pub fn export_entries(&self) -> Result<Vec<ExportEntry>> {
        let blocked = self.blocked_paths()?;
        let game_dir = self.game_dir();
        let read = match std::fs::read_dir(&game_dir) {
            Ok(read) => read,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(err) => return Err(Error::io(&game_dir, err)),
        };
        let mut entries: Vec<_> = read
            .flatten()
            .filter_map(|entry| {
                let path = entry.file_name().into_string().ok()?;
                (!IGNORED_NAMES.contains(&path.as_str())).then(|| ExportEntry {
                    is_dir: entry.path().is_dir(),
                    blocked: blocked.iter().any(|blocked| blocked == &path),
                    path,
                })
            })
            .collect();
        entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.path.cmp(&b.path)));
        Ok(entries)
    }

    pub fn blocked_paths(&self) -> Result<Vec<String>> {
        let path = self.root.join(PACK_IGNORE_FILE);
        match std::fs::read_to_string(&path) {
            Ok(text) => Ok(text
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(str::to_owned)
                .collect()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                Ok(DEFAULT_BLOCKED.map(str::to_owned).to_vec())
            }
            Err(err) => Err(Error::io(&path, err)),
        }
    }

    pub fn set_blocked_paths(&self, blocked: &[String]) -> Result<()> {
        crate::fs::write_atomic(
            &self.root.join(PACK_IGNORE_FILE),
            blocked.join("\n").as_bytes(),
        )
    }

    /// Game files to export, relative to the game directory.
    pub fn exported_files(&self, blocked: &[String]) -> Result<Vec<PathBuf>> {
        let game_dir = self.game_dir();
        if !game_dir.is_dir() {
            return Ok(Vec::new());
        }
        Ok(archive::walk_files(&game_dir)?
            .into_iter()
            .filter(|path| !blocked.iter().any(|blocked| path.starts_with(blocked)))
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_none_or(|name| !IGNORED_NAMES.contains(&name))
            })
            .collect())
    }
}

/// Writes the instance config and the selected game files into a zip that can be imported again.
pub fn export_instance(instance: &Instance, output: &Path, blocked: &[String]) -> Result<()> {
    instance.set_blocked_paths(blocked)?;
    let config = serde_json::to_vec_pretty(&instance.config)
        .map_err(|err| Error::json(INSTANCE_CONFIG_FILE, err))?;
    let mut zip = ZipBuilder::create(output)?;
    zip.add_bytes(Path::new(INSTANCE_CONFIG_FILE), &config)?;
    let game_dir = instance.game_dir();
    for relative in instance.exported_files(blocked)? {
        zip.add_file(
            &game_dir.join(&relative),
            &Path::new(GAME_DIR).join(relative),
        )?;
    }
    zip.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instance::{NewInstance, create_instance};

    fn instance_with_files(dir: &Path) -> Result<Instance> {
        let instance = create_instance(
            &dir.join("instances"),
            NewInstance {
                name: "Pack".into(),
                minecraft_version: "1.21.4".into(),
                loader: None,
                group: None,
                icon_key: None,
            },
        )?;
        for file in [
            "options.txt",
            "logs/latest.log",
            "config/a.toml",
            ".DS_Store",
        ] {
            let path = instance.game_dir().join(file);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|err| Error::io(parent, err))?;
            }
            std::fs::write(&path, file).map_err(|err| Error::io(&path, err))?;
        }
        Ok(instance)
    }

    #[test]
    fn lists_entries_with_default_blocks() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|err| Error::io("tempdir", err))?;
        let instance = instance_with_files(dir.path())?;
        let entries: Vec<_> = instance
            .export_entries()?
            .into_iter()
            .map(|entry| (entry.path, entry.blocked))
            .collect();
        assert_eq!(
            entries,
            [
                ("config".to_owned(), false),
                ("logs".to_owned(), true),
                ("options.txt".to_owned(), false)
            ]
        );
        Ok(())
    }

    #[test]
    fn exports_a_reimportable_archive_and_remembers_blocks() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|err| Error::io("tempdir", err))?;
        let instance = instance_with_files(dir.path())?;
        let output = dir.path().join("out/pack.zip");
        export_instance(&instance, &output, &["logs".into(), "config".into()])?;

        assert_eq!(
            archive::entry_names(&output)?,
            ["instance.json", "minecraft/options.txt"]
        );
        assert_eq!(instance.blocked_paths()?, ["logs", "config"]);
        Ok(())
    }
}
