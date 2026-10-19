mod mrpack;

use serde::{Deserialize, Serialize};

pub use mrpack::{MrpackIndex, install_mrpack, read_mrpack_index};

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
