mod mod_list;
mod mod_metadata;
mod pack_metadata;
mod relations;
mod sources;

use std::path::Path;
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};

pub use mod_list::{ModListFormat, format_mod_list};
pub use mod_metadata::read_mod_metadata;
pub use pack_metadata::read_pack_metadata;
pub use relations::{affected_by_toggle, attach_compatibility, attach_relations};
pub use sources::{
    ProjectInfo, ResourceSource, attach_sources, index_dir, is_source_entry, read_sources,
    remove_source, set_update_lock, write_source,
};

use crate::fs::child_path;
use crate::{Error, Result};

pub const DISABLED_SUFFIX: &str = ".disabled";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum ResourceKind {
    Mod,
    ResourcePack,
    ShaderPack,
    DataPack,
    Modpack,
}

impl ResourceKind {
    /// Folder inside the game directory this kind of resource is installed into.
    pub fn install_dir(self) -> &'static str {
        match self {
            Self::Mod => "mods",
            Self::ResourcePack => "resourcepacks",
            Self::ShaderPack => "shaderpacks",
            Self::DataPack => "datapacks",
            Self::Modpack => "",
        }
    }

    /// Packs may also be unpacked folders; mods are always archives.
    fn accepts_folders(self) -> bool {
        !matches!(self, Self::Mod)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ResourceMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub authors: Vec<String>,
    pub homepage: Option<String>,
    /// Ids of the mods this one declares as required.
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ResourceFile {
    pub file_name: String,
    pub enabled: bool,
    pub size: u64,
    /// Unix time in milliseconds.
    pub modified: i64,
    pub metadata: Option<ResourceMetadata>,
    /// The platform project the file was installed from, when known.
    pub source: Option<ResourceSource>,
    /// Enabled names of the listed files this one requires.
    pub requires: Vec<String>,
    /// Enabled names of the listed files that require this one.
    pub required_by: Vec<String>,
    /// The source does not list the instance's Minecraft version.
    pub incompatible: bool,
}

/// Lists installed resources of `kind` in `dir`, including disabled ones.
pub fn list_resources(dir: &Path, kind: ResourceKind) -> Result<Vec<ResourceFile>> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(Error::io(dir, err)),
    };
    let mut files: Vec<ResourceFile> = entries
        .flatten()
        .filter_map(|entry| resource_file(&entry.path(), kind))
        .collect();
    files.sort_by_key(|file| display_name(file).to_lowercase());
    Ok(files)
}

fn resource_file(path: &Path, kind: ResourceKind) -> Option<ResourceFile> {
    let file_name = path.file_name()?.to_str()?.to_owned();
    if file_name.starts_with('.') || is_source_entry(&file_name) {
        return None;
    }
    let base = file_name
        .strip_suffix(DISABLED_SUFFIX)
        .unwrap_or(&file_name);
    let is_archive = base.ends_with(".jar") || base.ends_with(".zip");
    let is_folder = path.is_dir();
    if !(is_archive && !is_folder || is_folder && kind.accepts_folders()) {
        return None;
    }
    let fs_metadata = std::fs::metadata(path).ok()?;
    let modified = fs_metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX));
    let metadata = match kind {
        ResourceKind::Mod => read_mod_metadata(path).ok().flatten(),
        ResourceKind::ResourcePack | ResourceKind::DataPack => read_pack_metadata(path, base),
        ResourceKind::ShaderPack | ResourceKind::Modpack => None,
    };
    Some(ResourceFile {
        enabled: !file_name.ends_with(DISABLED_SUFFIX),
        size: if is_folder { 0 } else { fs_metadata.len() },
        modified,
        metadata,
        source: None,
        requires: Vec::new(),
        required_by: Vec::new(),
        incompatible: false,
        file_name,
    })
}

fn display_name(file: &ResourceFile) -> &str {
    file.metadata.as_ref().map_or(&file.file_name, |m| &m.name)
}

pub fn set_resource_enabled(dir: &Path, file_name: &str, enabled: bool) -> Result<String> {
    let current = child_path(dir, file_name)?;
    let base = file_name.strip_suffix(DISABLED_SUFFIX).unwrap_or(file_name);
    let target_name = if enabled {
        base.to_owned()
    } else {
        format!("{base}{DISABLED_SUFFIX}")
    };
    let target = dir.join(&target_name);
    if current != target {
        std::fs::rename(&current, &target).map_err(|err| Error::io(&current, err))?;
    }
    Ok(target_name)
}

pub fn remove_resource(dir: &Path, file_name: &str) -> Result<()> {
    let path = child_path(dir, file_name)?;
    let removed = if path.is_dir() {
        std::fs::remove_dir_all(&path)
    } else {
        std::fs::remove_file(&path)
    };
    removed.map_err(|err| Error::io(&path, err))
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    fn write_zip(path: &Path, entry: &str, contents: &str) -> Result<()> {
        let file = std::fs::File::create(path).map_err(|err| Error::io(path, err))?;
        let mut zip = zip::ZipWriter::new(file);
        zip.start_file(entry, zip::write::SimpleFileOptions::default())
            .map_err(|err| Error::archive(path, err))?;
        zip.write_all(contents.as_bytes())
            .map_err(|err| Error::io(path, err))?;
        zip.finish().map_err(|err| Error::archive(path, err))?;
        Ok(())
    }

    #[test]
    fn lists_toggles_and_removes_mods() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|err| Error::io("tempdir", err))?;
        write_zip(
            &dir.path().join("sodium.jar"),
            "fabric.mod.json",
            r#"{"id":"sodium","name":"Sodium","version":"0.6"}"#,
        )?;
        std::fs::write(dir.path().join("notes.txt"), "x").map_err(|err| Error::io("notes", err))?;
        std::fs::create_dir(dir.path().join("folder")).map_err(|err| Error::io("folder", err))?;

        let mods = list_resources(dir.path(), ResourceKind::Mod)?;
        assert_eq!(mods.len(), 1);
        assert_eq!(
            mods[0].metadata.as_ref().map(|m| m.name.as_str()),
            Some("Sodium")
        );

        let disabled = set_resource_enabled(dir.path(), "sodium.jar", false)?;
        assert_eq!(disabled, "sodium.jar.disabled");
        assert!(!list_resources(dir.path(), ResourceKind::Mod)?[0].enabled);

        assert!(remove_resource(dir.path(), "../escape.jar").is_err());
        remove_resource(dir.path(), &disabled)?;
        assert!(list_resources(dir.path(), ResourceKind::Mod)?.is_empty());
        Ok(())
    }

    #[test]
    fn lists_zipped_and_folder_resource_packs() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|err| Error::io("tempdir", err))?;
        write_zip(
            &dir.path().join("faithful.zip"),
            "pack.mcmeta",
            r#"{"pack":{"pack_format":34,"description":"Faithful"}}"#,
        )?;
        let folder = dir.path().join("MyPack");
        std::fs::create_dir(&folder).map_err(|err| Error::io(&folder, err))?;
        std::fs::write(
            folder.join("pack.mcmeta"),
            r#"{"pack":{"pack_format":15,"description":"Mine"}}"#,
        )
        .map_err(|err| Error::io(&folder, err))?;

        let packs = list_resources(dir.path(), ResourceKind::ResourcePack)?;
        let descriptions: Vec<_> = packs
            .iter()
            .filter_map(|p| p.metadata.as_ref())
            .map(|m| m.description.as_str())
            .collect();
        assert_eq!(descriptions, ["Faithful", "Mine"]);

        remove_resource(dir.path(), "MyPack")?;
        assert!(!folder.exists());
        Ok(())
    }
}
