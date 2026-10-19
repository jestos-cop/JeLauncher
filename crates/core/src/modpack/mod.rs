mod import;
mod mrpack;
mod mrpack_export;

use std::path::Path;

use serde::{Deserialize, Serialize};

pub use import::{ArchiveKind, detect_archive, import_instance};
pub use mrpack::{MRPACK_INDEX, MrpackFile, MrpackIndex, install_mrpack, read_mrpack_index};
pub use mrpack_export::{MrpackExport, export_mrpack};

use crate::modplatform::Provider;

/// Where an instance created from a modpack came from, for updates and display.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ManagedPack {
    pub provider: Provider,
    pub project_id: String,
    pub version_id: String,
    pub name: String,
    pub version: String,
}

/// Identifies the published modpack version an archive was downloaded from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackSource {
    pub provider: Provider,
    pub project_id: String,
    pub version_id: String,
}

/// Where a downloaded or imported pack becomes a new instance.
pub struct ModpackTarget<'a> {
    pub instances_dir: &'a Path,
    pub download_dir: &'a Path,
    pub name: Option<String>,
    pub concurrency: usize,
}
