//! Where installed resources came from, stored as packwiz `.pw.toml` files next to them.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{DISABLED_SUFFIX, ResourceFile, ResourceKind};
use crate::modplatform::{ProjectVersion, Provider, ReleaseType, VersionFile};
use crate::{Error, Result, fs};

const INDEX_DIR: &str = ".index";
const METADATA_SUFFIX: &str = ".pw.toml";
const CURSEFORGE_MODE: &str = "metadata:curseforge";

/// The platform project and version an installed resource file was downloaded from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ResourceSource {
    pub provider: Provider,
    pub project_id: String,
    /// Modrinth version id or CurseForge file id.
    pub version_id: String,
    pub slug: String,
    pub name: String,
    /// The file name without the `.disabled` suffix.
    pub file_name: String,
    pub version_number: Option<String>,
    pub release_type: Option<ReleaseType>,
    /// Kept as it is: updates are left to the modpack the file came with.
    pub lock_update: bool,
    #[serde(skip)]
    download: Download,
    #[serde(skip)]
    side: Option<String>,
    #[serde(skip)]
    dependencies: Vec<String>,
}

/// The parts of a platform project a source needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectInfo {
    pub slug: String,
    pub name: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Download {
    url: String,
    hash_format: String,
    hash: String,
}

impl ResourceSource {
    /// The source of `file` from `version`, installed on disk as `file_name`.
    pub fn new(
        provider: Provider,
        project: &ProjectInfo,
        version: &ProjectVersion,
        file: &VersionFile,
        file_name: &str,
        lock_update: bool,
    ) -> Self {
        let (hash_format, hash) = match (&file.sha512, &file.sha1) {
            (Some(sha512), _) if provider == Provider::Modrinth => ("sha512", sha512.clone()),
            (_, Some(sha1)) => ("sha1", sha1.clone()),
            _ => ("sha1", String::new()),
        };
        Self {
            provider,
            project_id: version.project_id.clone(),
            version_id: version.id.clone(),
            slug: project.slug.clone(),
            name: project.name.clone(),
            file_name: enabled_name(file_name).to_owned(),
            version_number: Some(version.version_number.clone()),
            release_type: Some(version.release_type),
            lock_update,
            download: Download {
                url: if provider == Provider::CurseForge {
                    String::new()
                } else {
                    file.url.clone()
                },
                hash_format: hash_format.to_owned(),
                hash,
            },
            side: None,
            dependencies: version
                .dependencies
                .iter()
                .filter(|dependency| dependency.required)
                .filter_map(|dependency| dependency.project_id.clone())
                .collect(),
        }
    }

    /// The project page on the platform's website.
    pub fn project_url(&self) -> String {
        match self.provider {
            Provider::CurseForge => {
                format!("https://www.curseforge.com/projects/{}", self.project_id)
            }
            _ => format!("https://modrinth.com/project/{}", self.project_id),
        }
    }
}

/// Folder holding the source files of a resource folder. Shader packs keep them alongside,
/// because shader mods would list an index folder as a broken pack.
pub fn index_dir(dir: &Path, kind: ResourceKind) -> PathBuf {
    match kind {
        ResourceKind::ShaderPack => dir.to_path_buf(),
        _ => dir.join(INDEX_DIR),
    }
}

/// Whether `name` is source bookkeeping rather than a resource.
pub fn is_source_entry(name: &str) -> bool {
    name == INDEX_DIR || name.ends_with(METADATA_SUFFIX)
}

/// Every readable source file in the index of `dir`; broken files are skipped.
pub fn read_sources(dir: &Path, kind: ResourceKind) -> Vec<(PathBuf, ResourceSource)> {
    let Ok(entries) = std::fs::read_dir(index_dir(dir, kind)) else {
        return Vec::new();
    };
    entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(METADATA_SUFFIX))
        })
        .filter_map(|path| {
            let text = std::fs::read_to_string(&path).ok()?;
            let source = parse_source(&text, &path)?;
            Some((path, source))
        })
        .collect()
}

/// Writes the source of one installed file, replacing any older source of the same project
/// or for the same file.
pub fn write_source(dir: &Path, kind: ResourceKind, source: &ResourceSource) -> Result<()> {
    let index = index_dir(dir, kind);
    std::fs::create_dir_all(&index).map_err(|err| Error::io(&index, err))?;
    let path = index.join(format!("{}{METADATA_SUFFIX}", file_slug(&source.slug)));
    let text = toml::to_string(&PackwizFile::from(source))
        .map_err(|err| Error::InvalidInput(format!("cannot write {}: {err}", path.display())))?;
    fs::write_atomic(&path, text.as_bytes())?;
    for (other, existing) in read_sources(dir, kind) {
        let same_project =
            existing.provider == source.provider && existing.project_id == source.project_id;
        if other != path && (same_project || existing.file_name == source.file_name) {
            std::fs::remove_file(&other).map_err(|err| Error::io(&other, err))?;
        }
    }
    Ok(())
}

/// Removes the source of `file_name` (with or without `.disabled`), if it has one.
pub fn remove_source(dir: &Path, kind: ResourceKind, file_name: &str) -> Result<()> {
    let name = enabled_name(file_name);
    for (path, source) in read_sources(dir, kind) {
        if source.file_name == name {
            std::fs::remove_file(&path).map_err(|err| Error::io(&path, err))?;
        }
    }
    Ok(())
}

/// Turns update checks off or on for the resource `file_name`.
pub fn set_update_lock(
    dir: &Path,
    kind: ResourceKind,
    file_name: &str,
    locked: bool,
) -> Result<()> {
    let name = enabled_name(file_name);
    let Some((_, mut source)) = read_sources(dir, kind)
        .into_iter()
        .find(|(_, source)| source.file_name == name)
    else {
        return Err(Error::InvalidInput(format!(
            "{file_name} has no known source"
        )));
    };
    source.lock_update = locked;
    write_source(dir, kind, &source)
}

/// Attaches sources to listed files and deletes sources whose file no longer exists.
pub fn attach_sources(dir: &Path, kind: ResourceKind, files: &mut [ResourceFile]) -> Result<()> {
    let mut claimed = HashSet::new();
    let mut sources = read_sources(dir, kind);
    // Newest first, so a stale duplicate for the same file loses.
    sources.sort_by_key(|(path, _)| std::cmp::Reverse(modified(path)));
    for (path, source) in sources {
        let file = files
            .iter_mut()
            .find(|file| enabled_name(&file.file_name) == source.file_name);
        match file {
            Some(file) if claimed.insert(source.file_name.clone()) => file.source = Some(source),
            _ => std::fs::remove_file(&path).map_err(|err| Error::io(&path, err))?,
        }
    }
    Ok(())
}

pub(crate) fn enabled_name(file_name: &str) -> &str {
    file_name.strip_suffix(DISABLED_SUFFIX).unwrap_or(file_name)
}

/// The slug as a portable file name.
fn file_slug(slug: &str) -> String {
    let cleaned: String = slug
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
                c
            } else {
                '-'
            }
        })
        .collect();
    let cleaned = cleaned.trim_matches('.').to_owned();
    if cleaned.is_empty() {
        "resource".to_owned()
    } else {
        cleaned
    }
}

fn modified(path: &Path) -> std::time::SystemTime {
    std::fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .unwrap_or(std::time::UNIX_EPOCH)
}

fn parse_source(text: &str, path: &Path) -> Option<ResourceSource> {
    let file: PackwizFile = toml::from_str(text).ok()?;
    let (provider, project_id, version_id) = match (&file.update.curseforge, &file.update.modrinth)
    {
        (Some(curseforge), _) => (
            Provider::CurseForge,
            curseforge.project_id.to_string(),
            curseforge.file_id.to_string(),
        ),
        (None, Some(modrinth)) => (
            Provider::Modrinth,
            modrinth.mod_id.clone(),
            modrinth.version.clone(),
        ),
        (None, None) => return None,
    };
    let slug = path
        .file_name()?
        .to_str()?
        .strip_suffix(METADATA_SUFFIX)?
        .to_owned();
    Some(ResourceSource {
        provider,
        project_id,
        version_id,
        slug,
        name: file.name,
        file_name: file.filename,
        version_number: file.version_number,
        release_type: file.release_type,
        lock_update: file.lock_update,
        download: Download {
            url: file.download.url,
            hash_format: file.download.hash_format,
            hash: file.download.hash,
        },
        side: file.side,
        dependencies: file
            .dependencies
            .into_iter()
            .map(|dependency| dependency.project_id)
            .collect(),
    })
}

/// Packwiz v1 metadata, plus launcher fields under an `x-jelauncher-` prefix.
#[derive(Serialize, Deserialize)]
struct PackwizFile {
    name: String,
    filename: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    side: Option<String>,
    #[serde(
        rename = "x-jelauncher-version-number",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    version_number: Option<String>,
    #[serde(
        rename = "x-jelauncher-release-type",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    release_type: Option<ReleaseType>,
    #[serde(rename = "x-jelauncher-lock-update", default)]
    lock_update: bool,
    #[serde(
        rename = "x-jelauncher-dependencies",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    dependencies: Vec<PackwizDependency>,
    download: PackwizDownload,
    update: PackwizUpdate,
}

#[derive(Serialize, Deserialize)]
struct PackwizDependency {
    #[serde(rename = "project-id")]
    project_id: String,
    #[serde(rename = "type")]
    kind: String,
}

#[derive(Serialize, Deserialize)]
struct PackwizDownload {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    mode: Option<String>,
    #[serde(default)]
    url: String,
    #[serde(rename = "hash-format")]
    hash_format: String,
    hash: String,
}

#[derive(Serialize, Deserialize)]
struct PackwizUpdate {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    curseforge: Option<CurseForgeUpdate>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    modrinth: Option<ModrinthUpdate>,
}

#[derive(Serialize, Deserialize)]
struct CurseForgeUpdate {
    #[serde(rename = "file-id")]
    file_id: u64,
    #[serde(rename = "project-id")]
    project_id: u64,
}

#[derive(Serialize, Deserialize)]
struct ModrinthUpdate {
    #[serde(rename = "mod-id")]
    mod_id: String,
    version: String,
}

impl From<&ResourceSource> for PackwizFile {
    fn from(source: &ResourceSource) -> Self {
        let update = match source.provider {
            Provider::CurseForge => PackwizUpdate {
                curseforge: Some(CurseForgeUpdate {
                    file_id: source.version_id.parse().unwrap_or_default(),
                    project_id: source.project_id.parse().unwrap_or_default(),
                }),
                modrinth: None,
            },
            _ => PackwizUpdate {
                curseforge: None,
                modrinth: Some(ModrinthUpdate {
                    mod_id: source.project_id.clone(),
                    version: source.version_id.clone(),
                }),
            },
        };
        Self {
            name: source.name.clone(),
            filename: source.file_name.clone(),
            side: source.side.clone(),
            version_number: source.version_number.clone(),
            release_type: source.release_type,
            lock_update: source.lock_update,
            dependencies: source
                .dependencies
                .iter()
                .map(|project_id| PackwizDependency {
                    project_id: project_id.clone(),
                    kind: "required".to_owned(),
                })
                .collect(),
            download: PackwizDownload {
                mode: Some(
                    if source.provider == Provider::CurseForge {
                        CURSEFORGE_MODE
                    } else {
                        "url"
                    }
                    .to_owned(),
                ),
                url: source.download.url.clone(),
                hash_format: source.download.hash_format.clone(),
                hash: source.download.hash.clone(),
            },
            update,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modplatform::Dependency;

    fn version(provider: Provider) -> (ProjectVersion, VersionFile) {
        let file = VersionFile {
            url: "https://cdn.modrinth.com/data/AANobbMI/sodium.jar".into(),
            file_name: "sodium.jar".into(),
            sha1: Some("aa".into()),
            sha512: Some("bb".into()),
            size: 1,
            primary: true,
        };
        let version = ProjectVersion {
            id: if provider == Provider::CurseForge {
                "5123456"
            } else {
                "OihdIimA"
            }
            .into(),
            project_id: if provider == Provider::CurseForge {
                "394468"
            } else {
                "AANobbMI"
            }
            .into(),
            name: "Sodium 0.6.13".into(),
            version_number: "0.6.13".into(),
            release_type: ReleaseType::Release,
            published: String::new(),
            changelog: None,
            files: vec![file.clone()],
            dependencies: vec![Dependency {
                project_id: Some("P7dR8mSH".into()),
                version_id: None,
                required: true,
            }],
        };
        (version, file)
    }

    fn project() -> ProjectInfo {
        ProjectInfo {
            slug: "Sodium Mod!".into(),
            name: "Sodium".into(),
        }
    }

    fn tempdir() -> Result<tempfile::TempDir> {
        tempfile::tempdir().map_err(|err| Error::io("tempdir", err))
    }

    #[test]
    fn writes_and_reads_back_both_providers() -> Result<()> {
        let dir = tempdir()?;
        for provider in [Provider::Modrinth, Provider::CurseForge] {
            let (version, file) = version(provider);
            let source = ResourceSource::new(
                provider,
                &project(),
                &version,
                &file,
                "sodium.jar.disabled",
                false,
            );
            write_source(dir.path(), ResourceKind::Mod, &source)?;
            let read = read_sources(dir.path(), ResourceKind::Mod);
            assert_eq!(read.len(), 1, "one file per project and file");
            let (path, back) = &read[0];
            assert!(path.ends_with(".index/sodium-mod-.pw.toml"));
            assert_eq!(back.provider, provider);
            assert_eq!(back.project_id, source.project_id);
            assert_eq!(back.version_id, source.version_id);
            assert_eq!(back.file_name, "sodium.jar");
            assert_eq!(back.dependencies, ["P7dR8mSH"]);
        }
        Ok(())
    }

    #[test]
    fn reads_plain_packwiz_files() -> Result<()> {
        let dir = tempdir()?;
        let index = dir.path().join(".index");
        std::fs::create_dir(&index).map_err(|err| Error::io(&index, err))?;
        let file = index.join("screenshot-to-clipboard.pw.toml");
        std::fs::write(
            &file,
            r#"name = "Screenshot to Clipboard (Fabric)"
filename = "screenshot-to-clipboard-1.0.7-fabric.jar"
side = "both"

[download]
url = "https://edge.forgecdn.net/files/3509/43/screenshot-to-clipboard-1.0.7-fabric.jar"
hash-format = "murmur2"
hash = "1781245820"

[update]
[update.curseforge]
file-id = 3509043
project-id = 327154
"#,
        )
        .map_err(|err| Error::io(&file, err))?;
        std::fs::write(index.join("broken.pw.toml"), "name = ")
            .map_err(|err| Error::io(&index, err))?;
        let read = read_sources(dir.path(), ResourceKind::Mod);
        assert_eq!(read.len(), 1);
        assert_eq!(read[0].1.provider, Provider::CurseForge);
        assert_eq!(read[0].1.project_id, "327154");
        assert_eq!(read[0].1.version_id, "3509043");
        Ok(())
    }

    #[test]
    fn attaches_to_disabled_files_and_cleans_orphans() -> Result<()> {
        let dir = tempdir()?;
        let (version, file) = version(Provider::Modrinth);
        let source = ResourceSource::new(
            Provider::Modrinth,
            &project(),
            &version,
            &file,
            "sodium.jar",
            false,
        );
        write_source(dir.path(), ResourceKind::Mod, &source)?;
        let mut orphan = source.clone();
        orphan.project_id = "other".into();
        orphan.slug = "gone".into();
        orphan.file_name = "gone.jar".into();
        write_source(dir.path(), ResourceKind::Mod, &orphan)?;

        let mut files = vec![ResourceFile {
            file_name: "sodium.jar.disabled".into(),
            enabled: false,
            size: 1,
            modified: 0,
            metadata: None,
            source: None,
        }];
        attach_sources(dir.path(), ResourceKind::Mod, &mut files)?;
        assert_eq!(
            files[0].source.as_ref().map(|s| s.project_id.as_str()),
            Some("AANobbMI")
        );
        assert_eq!(read_sources(dir.path(), ResourceKind::Mod).len(), 1);

        set_update_lock(dir.path(), ResourceKind::Mod, "sodium.jar.disabled", true)?;
        assert!(read_sources(dir.path(), ResourceKind::Mod)[0].1.lock_update);
        remove_source(dir.path(), ResourceKind::Mod, "sodium.jar.disabled")?;
        assert!(read_sources(dir.path(), ResourceKind::Mod).is_empty());
        Ok(())
    }

    #[test]
    fn shader_pack_sources_sit_next_to_the_packs() {
        assert_eq!(
            index_dir(Path::new("/s"), ResourceKind::ShaderPack),
            Path::new("/s")
        );
        assert_eq!(
            index_dir(Path::new("/m"), ResourceKind::Mod),
            Path::new("/m/.index")
        );
        assert!(is_source_entry(".index"));
        assert!(is_source_entry("complementary.pw.toml"));
    }
}
