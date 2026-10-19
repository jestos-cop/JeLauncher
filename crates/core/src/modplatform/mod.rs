pub mod modrinth;

use serde::{Deserialize, Serialize};

use crate::minecraft::LoaderKind;
use crate::net::Download;
pub use crate::resources::ResourceKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum Provider {
    Modrinth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum SortOrder {
    #[default]
    Relevance,
    Downloads,
    Updated,
    Newest,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchQuery {
    pub text: String,
    pub kind: ResourceKind,
    pub game_version: Option<String>,
    pub loader: Option<LoaderKind>,
    pub sort: SortOrder,
    pub offset: u32,
    pub limit: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchPage {
    pub hits: Vec<ProjectHit>,
    pub total: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ProjectHit {
    pub provider: Provider,
    pub id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub author: String,
    pub icon_url: Option<String>,
    pub downloads: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum ReleaseType {
    Release,
    Beta,
    Alpha,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ProjectVersion {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub version_number: String,
    pub release_type: ReleaseType,
    pub published: String,
    /// Markdown release notes.
    pub changelog: Option<String>,
    pub files: Vec<VersionFile>,
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct VersionFile {
    pub url: String,
    pub file_name: String,
    pub sha1: Option<String>,
    pub sha512: Option<String>,
    pub size: u64,
    pub primary: bool,
}

/// A newer published version of an installed resource file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ResourceUpdate {
    pub file_name: String,
    pub version: ProjectVersion,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Dependency {
    pub project_id: Option<String>,
    pub version_id: Option<String>,
    pub required: bool,
}

impl ProjectVersion {
    /// The file to install: the one marked primary, otherwise the first.
    pub fn primary_file(&self) -> Option<&VersionFile> {
        self.files
            .iter()
            .find(|file| file.primary)
            .or_else(|| self.files.first())
    }
}

impl VersionFile {
    pub fn download_into(&self, dir: &std::path::Path) -> Download {
        Download {
            url: self.url.clone(),
            path: dir.join(&self.file_name),
            sha1: self.sha1.clone(),
            size: Some(self.size).filter(|&size| size > 0),
        }
    }
}
