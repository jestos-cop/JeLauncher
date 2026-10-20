use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::export_files::{OVERRIDES_DIR, entry_name, is_resolvable, is_source_metadata};
use super::{MRPACK_INDEX, MrpackFile, MrpackIndex};
use crate::archive::ZipBuilder;
use crate::instance::Instance;
use crate::modplatform::ProjectVersion;
use crate::modplatform::modrinth::Modrinth;
use crate::net::sha1_of;
use crate::{Error, Result};

pub struct MrpackExport<'a> {
    pub output: &'a Path,
    pub version: String,
    pub summary: Option<String>,
    pub blocked: &'a [String],
}

/// Writes a `.mrpack`: files found on Modrinth become downloads, everything else an override.
pub async fn export_mrpack(
    modrinth: &Modrinth,
    instance: &Instance,
    options: &MrpackExport<'_>,
) -> Result<()> {
    instance.set_blocked_paths(options.blocked)?;
    let game_dir = instance.game_dir();
    let files: Vec<_> = instance
        .exported_files(options.blocked)?
        .into_iter()
        .filter(|path| !is_source_metadata(path))
        .collect();
    let mut by_hash = HashMap::new();
    for relative in files.iter().filter(|path| is_resolvable(path)) {
        by_hash.insert(sha1_of(&game_dir.join(relative)).await?, relative.clone());
    }
    let hashes: Vec<String> = by_hash.keys().cloned().collect();
    let versions = modrinth.versions_by_sha1(&hashes).await?;
    let mut resolved: Vec<_> = versions
        .iter()
        .filter_map(|(hash, version)| Some((by_hash.get(hash)?, published_file(hash, version)?)))
        .collect();
    resolved.sort_by(|a, b| a.0.cmp(b.0));
    let resolved_paths: HashSet<&PathBuf> = resolved.iter().map(|(path, _)| *path).collect();

    let index = MrpackIndex {
        format_version: 1,
        game: "minecraft".into(),
        name: instance.config.name.clone(),
        version_id: options.version.clone(),
        summary: options.summary.clone(),
        files: resolved
            .iter()
            .map(|(path, file)| MrpackFile {
                path: entry_name(path),
                ..file.clone()
            })
            .collect(),
        dependencies: MrpackIndex::dependencies_for(
            &instance.config.minecraft_version,
            instance.config.loader.as_ref(),
        ),
    };
    let json = serde_json::to_vec_pretty(&index).map_err(|err| Error::json(MRPACK_INDEX, err))?;
    let mut zip = ZipBuilder::create(options.output)?;
    zip.add_bytes(Path::new(MRPACK_INDEX), &json)?;
    for relative in files.iter().filter(|path| !resolved_paths.contains(path)) {
        zip.add_file(
            &game_dir.join(relative),
            &Path::new(OVERRIDES_DIR).join(relative),
        )?;
    }
    zip.finish()
}

fn published_file(sha1: &str, version: &ProjectVersion) -> Option<MrpackFile> {
    let file = version
        .files
        .iter()
        .find(|file| file.sha1.as_deref() == Some(sha1))?;
    Some(MrpackFile {
        path: String::new(),
        hashes: HashMap::from([
            ("sha1".to_owned(), sha1.to_owned()),
            ("sha512".to_owned(), file.sha512.clone()?),
        ]),
        env: None,
        downloads: vec![file.url.clone()],
        file_size: file.size,
    })
}
