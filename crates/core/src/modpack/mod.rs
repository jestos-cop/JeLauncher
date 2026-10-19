mod curseforge_pack;
mod ftb_pack;
mod import;
mod mrpack;
mod mrpack_export;
mod technic_pack;

use std::path::Path;

use serde::{Deserialize, Serialize};

pub use curseforge_pack::{
    BlockedFile, CURSEFORGE_MANIFEST, CurseManifest, install_curseforge_pack,
    read_curseforge_manifest,
};
pub use ftb_pack::install_ftb_pack;
pub use import::{ArchiveKind, detect_archive, import_instance};
pub use mrpack::{
    MRPACK_INDEX, MrpackFile, MrpackIndex, install_mrpack, read_mrpack_index, update_mrpack,
};
pub use mrpack_export::{MrpackExport, export_mrpack};
pub use technic_pack::install_technic_pack;

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
    /// Lets `.mrpack` files download from hosts outside the platform allow list.
    pub allow_untrusted_hosts: bool,
}

/// A newly created pack instance plus the files the user still has to download by hand.
#[derive(Debug, Clone)]
pub struct PackInstall {
    pub instance: crate::instance::Instance,
    pub blocked: Vec<BlockedFile>,
}

impl PackInstall {
    pub fn complete(instance: crate::instance::Instance) -> Self {
        Self {
            instance,
            blocked: Vec::new(),
        }
    }
}
