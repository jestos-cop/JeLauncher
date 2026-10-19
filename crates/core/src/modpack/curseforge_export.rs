use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::export_files::{OVERRIDES_DIR, is_resolvable};
use super::{CURSEFORGE_MANIFEST, CurseManifest, ManifestFile, ManifestLoader, ManifestMinecraft};
use crate::archive::ZipBuilder;
use crate::instance::Instance;
use crate::modplatform::curseforge::{CurseForge, fingerprint};
use crate::{Error, Result};

pub struct CurseForgeExport<'a> {
    pub output: &'a Path,
    pub version: String,
    pub author: String,
    pub blocked: &'a [String],
}

/// Writes a CurseForge pack: files CurseForge recognises are listed in the manifest,
/// everything else is bundled as an override.
pub async fn export_curseforge_pack(
    curseforge: &CurseForge,
    instance: &Instance,
    options: &CurseForgeExport<'_>,
) -> Result<()> {
    instance.set_blocked_paths(options.blocked)?;
    let game_dir = instance.game_dir();
    let files = instance.exported_files(options.blocked)?;
    let mut by_fingerprint = HashMap::new();
    for relative in files.iter().filter(|path| is_resolvable(path)) {
        let path = game_dir.join(relative);
        let bytes = tokio::fs::read(&path)
            .await
            .map_err(|err| Error::io(&path, err))?;
        by_fingerprint.insert(fingerprint(&bytes), relative.clone());
    }
    let fingerprints: Vec<u32> = by_fingerprint.keys().copied().collect();
    let matches = curseforge.match_fingerprints(&fingerprints).await?;
    let mut resolved: Vec<(&PathBuf, ManifestFile)> = matches
        .iter()
        .filter_map(|(fingerprint, (project_id, file_id))| {
            let path = by_fingerprint.get(fingerprint)?;
            Some((
                path,
                ManifestFile {
                    project_id: *project_id,
                    file_id: *file_id,
                    required: true,
                },
            ))
        })
        .collect();
    resolved.sort_by(|a, b| a.0.cmp(b.0));
    let resolved_paths: HashSet<&PathBuf> = resolved.iter().map(|(path, _)| *path).collect();

    let manifest = CurseManifest {
        minecraft: ManifestMinecraft {
            version: instance.config.minecraft_version.clone(),
            mod_loaders: instance
                .config
                .loader
                .iter()
                .map(|loader| ManifestLoader {
                    id: format!("{}-{}", loader.kind.id(), loader.version),
                    primary: true,
                })
                .collect(),
        },
        manifest_type: "minecraftModpack".into(),
        manifest_version: 1,
        name: instance.config.name.clone(),
        version: options.version.clone(),
        author: options.author.clone(),
        files: resolved.into_iter().map(|(_, file)| file).collect(),
        overrides: OVERRIDES_DIR.into(),
    };
    let json = serde_json::to_vec_pretty(&manifest)
        .map_err(|err| Error::json(CURSEFORGE_MANIFEST, err))?;
    let mut zip = ZipBuilder::create(options.output)?;
    zip.add_bytes(Path::new(CURSEFORGE_MANIFEST), &json)?;
    for relative in files.iter().filter(|path| !resolved_paths.contains(path)) {
        zip.add_file(
            &game_dir.join(relative),
            &Path::new(OVERRIDES_DIR).join(relative),
        )?;
    }
    zip.finish()
}
