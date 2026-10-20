pub mod atlauncher;
pub mod curseforge;
pub mod ftb;
pub mod modrinth;
mod platforms;
pub mod technic;

use serde::{Deserialize, Serialize};

use crate::minecraft::LoaderKind;
use std::path::Path;

pub use platforms::{ModPlatforms, PlatformConfig};

use crate::net::{Download, HttpClient};
pub use crate::resources::ResourceKind;
use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum Provider {
    #[default]
    Modrinth,
    CurseForge,
    Ftb,
    Technic,
    AtLauncher,
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

impl Provider {
    /// Search orders this platform can apply; the first one is the default.
    pub fn sort_orders(self) -> &'static [SortOrder] {
        match self {
            Self::Modrinth | Self::CurseForge => &[
                SortOrder::Relevance,
                SortOrder::Downloads,
                SortOrder::Updated,
                SortOrder::Newest,
            ],
            Self::Ftb => &[
                SortOrder::Relevance,
                SortOrder::Downloads,
                SortOrder::Updated,
            ],
            Self::Technic | Self::AtLauncher => &[SortOrder::Relevance],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchQuery {
    #[serde(default)]
    pub provider: Provider,
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

/// Where and for which game setup resources are installed.
pub struct InstallTarget<'a> {
    pub dir: &'a Path,
    pub kind: ResourceKind,
    pub game_version: &'a str,
    pub loader: Option<LoaderKind>,
}

/// Downloads the version's primary file into `dir` and returns the file name.
pub async fn download_version(
    http: &HttpClient,
    version: &ProjectVersion,
    dir: &Path,
) -> Result<String> {
    let file = version
        .primary_file()
        .ok_or_else(|| Error::InvalidInput(format!("version {} has no files", version.name)))?;
    if file.url.is_empty() {
        return Err(Error::InvalidInput(format!(
            "the author of {} does not allow downloads through other launchers; download it from the project page",
            file.file_name
        )));
    }
    http.download(&file.download_into(dir)).await?;
    Ok(file.file_name.clone())
}
