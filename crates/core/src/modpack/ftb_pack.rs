use super::{ManagedPack, ModpackTarget, PackInstall};
use crate::archive::safe_relative_path;
use crate::instance::{NewInstance, create_instance};
use crate::modplatform::Provider;
use crate::modplatform::ftb::{Ftb, FtbPackVersion};
use crate::net::{Download, DownloadProgress, HttpClient};
use crate::{Error, Result};

/// Creates a new instance from an FTB pack version by downloading all of its client files.
pub async fn install_ftb_pack(
    ftb: &Ftb,
    http: &HttpClient,
    pack_id: &str,
    version_id: &str,
    target: &ModpackTarget<'_>,
    on_progress: impl Fn(DownloadProgress),
) -> Result<PackInstall> {
    let pack = ftb.pack_version(pack_id, version_id).await?;
    let mut instance = create_instance(
        target.instances_dir,
        NewInstance {
            name: target
                .name
                .clone()
                .unwrap_or_else(|| pack.pack_name.clone()),
            minecraft_version: pack.minecraft_version.clone(),
            loader: pack.loader.clone(),
            group: None,
            icon_key: None,
        },
    )?;
    let downloaded = match downloads(&pack, &instance.game_dir()) {
        Ok(downloads) => {
            http.download_all(downloads, target.concurrency, on_progress)
                .await
        }
        Err(err) => Err(err),
    };
    if let Err(err) = downloaded {
        let _ = std::fs::remove_dir_all(&instance.root);
        return Err(err);
    }
    instance.config.managed_pack = Some(ManagedPack {
        provider: Provider::Ftb,
        project_id: pack_id.to_owned(),
        version_id: version_id.to_owned(),
        name: pack.pack_name,
        version: pack.version_name,
    });
    instance.save()?;
    Ok(PackInstall::complete(instance))
}

fn downloads(pack: &FtbPackVersion, game_dir: &std::path::Path) -> Result<Vec<Download>> {
    pack.files
        .iter()
        .map(|file| {
            let relative = safe_relative_path(&file.path).ok_or_else(|| {
                Error::InvalidInput(format!("unsafe path in pack: {}", file.path))
            })?;
            Ok(Download {
                url: file.url.clone(),
                path: game_dir.join(relative),
                sha1: file.sha1.clone(),
                size: Some(file.size).filter(|&size| size > 0),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::modplatform::ftb::FtbFile;

    fn pack(path: &str) -> FtbPackVersion {
        FtbPackVersion {
            pack_name: "Pack".into(),
            version_name: "1.0".into(),
            minecraft_version: "1.18.2".into(),
            loader: None,
            files: vec![FtbFile {
                path: path.into(),
                url: "https://x/a".into(),
                sha1: None,
                size: 0,
            }],
        }
    }

    #[test]
    fn places_files_inside_the_game_directory() -> Result<()> {
        let placed = downloads(&pack("mods/a.jar"), Path::new("/game"))?;
        assert_eq!(placed[0].path, Path::new("/game/mods/a.jar"));
        assert!(downloads(&pack("../evil.jar"), Path::new("/game")).is_err());
        Ok(())
    }
}
