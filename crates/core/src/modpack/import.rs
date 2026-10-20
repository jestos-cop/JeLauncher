use std::path::{Path, PathBuf};

use sha1::{Digest, Sha1};

use super::{
    CURSEFORGE_MANIFEST, MRPACK_INDEX, ModpackTarget, PackInstall, install_curseforge_pack,
    install_mrpack,
};
use crate::archive;
use crate::instance::{
    DraftGuard, INSTANCE_CONFIG_FILE, Instance, InstanceConfig, NewInstance, create_instance,
};
use crate::modplatform::ModPlatforms;
use crate::net::{Download, DownloadProgress};
use crate::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArchiveKind {
    Mrpack,
    CurseForge,
    /// An exported instance; `config_entry` is the archive entry holding its config.
    Native {
        config_entry: String,
    },
}

pub fn detect_archive(archive_path: &Path) -> Result<ArchiveKind> {
    let names = archive::entry_names(archive_path)?;
    if names.iter().any(|name| name == MRPACK_INDEX) {
        return Ok(ArchiveKind::Mrpack);
    }
    if names.iter().any(|name| name == CURSEFORGE_MANIFEST) {
        return Ok(ArchiveKind::CurseForge);
    }
    names
        .into_iter()
        .filter(|name| Path::new(name).file_name() == Some(INSTANCE_CONFIG_FILE.as_ref()))
        .min_by_key(|name| Path::new(name).components().count())
        .map(|config_entry| ArchiveKind::Native { config_entry })
        .ok_or_else(|| {
            Error::InvalidInput(
                "unsupported archive: expected a .mrpack, a CurseForge pack or an exported instance".into(),
            )
        })
}

/// Imports an instance from a local archive path or an http(s) URL.
pub async fn import_instance(
    platforms: &ModPlatforms,
    source: &str,
    target: &ModpackTarget<'_>,
    on_progress: impl Fn(DownloadProgress),
) -> Result<PackInstall> {
    if !(source.starts_with("https://") || source.starts_with("http://")) {
        return import_archive(platforms, Path::new(source), target, on_progress).await;
    }
    let archive_path = target
        .download_dir
        .join(format!("import-{}.zip", hex::encode(Sha1::digest(source))));
    platforms
        .http()
        .download(&Download {
            url: source.to_owned(),
            path: archive_path.clone(),
            sha1: None,
            size: None,
        })
        .await?;
    let imported = import_archive(platforms, &archive_path, target, on_progress).await;
    let _ = std::fs::remove_file(&archive_path);
    imported
}

async fn import_archive(
    platforms: &ModPlatforms,
    archive_path: &Path,
    target: &ModpackTarget<'_>,
    on_progress: impl Fn(DownloadProgress),
) -> Result<PackInstall> {
    let installed = match detect_archive(archive_path)? {
        ArchiveKind::Mrpack => {
            install_mrpack(platforms.http(), archive_path, target, None, on_progress)
                .await
                .map(PackInstall::complete)?
        }
        ArchiveKind::CurseForge => {
            install_curseforge_pack(platforms, archive_path, target, None, on_progress).await?
        }
        // Exported instances carry their sources with them.
        ArchiveKind::Native { config_entry } => {
            return import_native(archive_path, &config_entry, target).map(PackInstall::complete);
        }
    };
    if target.track_sources {
        platforms
            .identify_pack_sources(&installed.instance, target.source_order)
            .await;
    }
    Ok(installed)
}

fn import_native(
    archive_path: &Path,
    config_entry: &str,
    target: &ModpackTarget<'_>,
) -> Result<Instance> {
    let text = archive::read_entry_to_string(archive_path, config_entry)?;
    let config: InstanceConfig =
        serde_json::from_str(&text).map_err(|err| Error::json(archive_path, err))?;
    let mut instance = create_instance(
        target.instances_dir,
        NewInstance {
            name: target.name.clone().unwrap_or_else(|| config.name.clone()),
            minecraft_version: config.minecraft_version.clone(),
            loader: config.loader.clone(),
            group: config.group.clone(),
            icon_key: Some(config.icon_key.clone()),
        },
    )?;
    let prefix = Path::new(config_entry)
        .parent()
        .unwrap_or(Path::new(""))
        .to_path_buf();
    let draft = DraftGuard::new(&instance);
    archive::extract(archive_path, &instance.root, |name| {
        name.strip_prefix(&prefix)
            .ok()
            .filter(|relative| *relative != Path::new(INSTANCE_CONFIG_FILE))
            .map(PathBuf::from)
    })?;
    instance.config = InstanceConfig {
        name: instance.config.name,
        group: instance.config.group,
        last_launch_time: 0,
        ..config
    };
    instance.save()?;
    draft.keep();
    Ok(instance)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    fn write_zip(path: &Path, entries: &[(&str, &str)]) -> Result<()> {
        let file = std::fs::File::create(path).map_err(|err| Error::io(path, err))?;
        let mut zip = zip::ZipWriter::new(file);
        for (name, content) in entries {
            zip.start_file(*name, zip::write::SimpleFileOptions::default())
                .map_err(|err| Error::archive(path, err))?;
            zip.write_all(content.as_bytes())
                .map_err(|err| Error::io(path, err))?;
        }
        zip.finish().map_err(|err| Error::archive(path, err))?;
        Ok(())
    }

    #[test]
    fn detects_archive_kinds() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|err| Error::io("tempdir", err))?;
        let mrpack = dir.path().join("pack.mrpack");
        write_zip(&mrpack, &[(MRPACK_INDEX, "{}")])?;
        assert_eq!(detect_archive(&mrpack)?, ArchiveKind::Mrpack);

        let native = dir.path().join("native.zip");
        write_zip(
            &native,
            &[
                ("My Pack/minecraft/config/instance.json", "{}"),
                ("My Pack/instance.json", "{}"),
            ],
        )?;
        assert_eq!(
            detect_archive(&native)?,
            ArchiveKind::Native {
                config_entry: "My Pack/instance.json".into()
            }
        );

        let other = dir.path().join("other.zip");
        write_zip(&other, &[("readme.txt", "hi")])?;
        assert!(detect_archive(&other).is_err());
        Ok(())
    }

    #[tokio::test]
    async fn imports_an_exported_instance_under_a_new_name() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|err| Error::io("tempdir", err))?;
        let archive_path = dir.path().join("export.zip");
        write_zip(
            &archive_path,
            &[
                (
                    "pack/instance.json",
                    r#"{"name":"Pack","minecraftVersion":"1.21.4","notes":"hi","totalTimePlayed":60}"#,
                ),
                ("pack/minecraft/options.txt", "fov:90"),
            ],
        )?;
        let instances = dir.path().join("instances");
        let target = ModpackTarget {
            instances_dir: &instances,
            download_dir: dir.path(),
            name: Some("Copy".into()),
            concurrency: 1,
            allow_untrusted_hosts: false,
            optional_mods: None,
            track_sources: false,
            source_order: Default::default(),
        };
        let platforms = ModPlatforms::new(
            crate::net::HttpClient::new(&crate::settings::NetworkSettings::default())?,
            Default::default(),
        );
        let instance =
            import_instance(&platforms, &archive_path.to_string_lossy(), &target, |_| {})
                .await?
                .instance;

        let reopened = Instance::open(&instances, &instance.id)?;
        assert_eq!(reopened.config.name, "Copy");
        assert_eq!(reopened.config.minecraft_version, "1.21.4");
        assert_eq!(reopened.config.notes, "hi");
        assert_eq!(reopened.config.total_time_played, 60);
        let options = reopened.game_dir().join("options.txt");
        assert_eq!(
            std::fs::read_to_string(&options).map_err(|err| Error::io(&options, err))?,
            "fov:90"
        );
        Ok(())
    }
}
