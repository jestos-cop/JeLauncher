use std::path::{Path, PathBuf};

use super::{BlockedFile, ManagedPack, ModpackTarget, PackInstall};
use crate::archive::{self, safe_relative_path};
use crate::instance::{DraftGuard, Instance, NewInstance, create_instance};
use crate::modplatform::Provider;
use crate::modplatform::atlauncher::{AtLauncher, AtlMod, AtlPackVersion, DOWNLOAD_SERVER};
use crate::net::{Download, DownloadProgress, HttpClient, verify_md5};
use crate::{Error, Result};

/// Creates a new instance from an ATLauncher pack version: its config archive, the chosen
/// optional mods and everything else the pack lists for the client.
pub async fn install_atlauncher_pack(
    atlauncher: &AtLauncher,
    http: &HttpClient,
    pack_id: &str,
    version: &str,
    target: &ModpackTarget<'_>,
    on_progress: impl Fn(DownloadProgress),
) -> Result<PackInstall> {
    let pack = atlauncher.pack_version(pack_id, version).await?;
    let pack_name = atlauncher.pack_name(pack_id).await?;
    let mut instance = create_instance(
        target.instances_dir,
        NewInstance {
            name: target.name.clone().unwrap_or_else(|| pack_name.clone()),
            minecraft_version: pack.minecraft.clone(),
            loader: pack.loader_spec()?,
            group: None,
            icon_key: None,
        },
    )?;
    let draft = DraftGuard::new(&instance);
    let staging = target
        .download_dir
        .join(format!("atlauncher-{}", instance.id));
    let populated = populate(
        http,
        &pack,
        pack_id,
        &staging,
        &instance,
        target,
        on_progress,
    )
    .await;
    let _ = std::fs::remove_dir_all(&staging);
    let blocked = populated?;
    instance.config.managed_pack = Some(ManagedPack {
        provider: Provider::AtLauncher,
        project_id: pack_id.to_owned(),
        version_id: version.to_owned(),
        name: pack_name,
        version: pack.version.clone(),
    });
    instance.save()?;
    draft.keep();
    Ok(PackInstall { instance, blocked })
}

/// One file to download and where it ends up.
struct PlannedMod<'a> {
    module: &'a AtlMod,
    download: Download,
}

async fn populate(
    http: &HttpClient,
    pack: &AtlPackVersion,
    pack_id: &str,
    staging: &Path,
    instance: &Instance,
    target: &ModpackTarget<'_>,
    on_progress: impl Fn(DownloadProgress),
) -> Result<Vec<BlockedFile>> {
    let game_dir = instance.game_dir();
    let configs = staging.join("Configs.zip");
    let mut downloads = Vec::new();
    if let (false, Some(configs_info)) = (pack.no_configs, &pack.configs) {
        downloads.push(Download {
            url: format!(
                "{DOWNLOAD_SERVER}/packs/{pack_id}/versions/{}/Configs.zip",
                pack.version
            ),
            path: configs.clone(),
            sha1: configs_info.sha1.clone(),
            size: None,
        });
    }
    let mut planned = Vec::new();
    let mut blocked = Vec::new();
    for (index, module) in pack
        .mods_to_install(target.optional_mods.as_deref())
        .into_iter()
        .enumerate()
    {
        let url = match module.download.as_str() {
            "server" => format!("{DOWNLOAD_SERVER}/{}", module.url.trim_start_matches('/')),
            "direct" => module.url.clone(),
            "browser" => {
                blocked.push(BlockedFile {
                    project_name: module.name.clone(),
                    file_name: module.file.clone(),
                    url: module.url.clone(),
                    install_dir: install_dir(module, &pack.minecraft)?
                        .map(|dir| dir.to_string_lossy().into_owned())
                        .unwrap_or_default(),
                });
                continue;
            }
            other => {
                return Err(Error::InvalidInput(format!(
                    "unknown download type {other} for {}",
                    module.name
                )));
            }
        };
        let download = Download {
            url,
            path: staging.join(format!("{index}-{}", sanitized(&module.file))),
            sha1: None,
            size: module.filesize,
        };
        downloads.push(download.clone());
        planned.push(PlannedMod { module, download });
    }
    http.download_all(downloads, target.concurrency, on_progress)
        .await?;

    if configs.is_file() {
        archive::extract(&configs, &game_dir, |name| Some(PathBuf::from(name)))?;
    }
    for PlannedMod { module, download } in planned {
        if !module.md5.is_empty() {
            verify_md5(&download.path, &module.md5)?;
        }
        place(module, &download.path, &game_dir, &pack.minecraft)?;
    }
    Ok(blocked)
}

/// Copies or extracts a downloaded mod file to where its type says it belongs.
fn place(module: &AtlMod, file: &Path, game_dir: &Path, minecraft: &str) -> Result<()> {
    let extract_into = |dir: PathBuf, folder: &str| {
        let folder = PathBuf::from(folder.trim_start_matches('/'));
        let dest = game_dir.join(dir);
        archive::extract(file, &dest, |name| {
            name.strip_prefix(&folder).ok().map(PathBuf::from)
        })
    };
    match module.kind.as_str() {
        "extract" => {
            let dir = named_dir(module.extract_to.as_deref().unwrap_or("root"), minecraft)?
                .unwrap_or_default();
            extract_into(dir, module.extract_folder.as_deref().unwrap_or(""))
        }
        "texturepackextract" => extract_into(PathBuf::from("texturepacks/extracted"), ""),
        "resourcepackextract" => extract_into(PathBuf::from("resourcepacks/extracted"), ""),
        "decomp" => {
            let wanted = module.decomp_file.clone().unwrap_or_default();
            let dir = named_dir(module.decomp_type.as_deref().unwrap_or("root"), minecraft)?
                .unwrap_or_default();
            archive::extract(file, &game_dir.join(dir), |name| {
                (name == Path::new(&wanted)).then(|| PathBuf::from(name))
            })
        }
        _ => match install_dir(module, minecraft)? {
            Some(dir) => {
                let relative = safe_relative_path(&module.file).ok_or_else(|| {
                    Error::InvalidInput(format!("unsafe path in pack: {}", module.file))
                })?;
                let dest = game_dir.join(dir).join(relative);
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent).map_err(|err| Error::io(parent, err))?;
                }
                std::fs::copy(file, &dest)
                    .map(drop)
                    .map_err(|err| Error::io(&dest, err))
            }
            None => {
                log::warn!("skipping {} of type {}", module.name, module.kind);
                Ok(())
            }
        },
    }
}

/// Folder inside the game directory for a plain mod file; `None` for types that are not copied.
fn install_dir(module: &AtlMod, minecraft: &str) -> Result<Option<PathBuf>> {
    named_dir(&module.kind, minecraft)
}

fn named_dir(kind: &str, minecraft: &str) -> Result<Option<PathBuf>> {
    let dir = match kind {
        "mods" => "mods".to_owned(),
        "flan" => "Flan".to_owned(),
        "dependency" => format!("mods/{minecraft}"),
        "ic2lib" => "mods/ic2".to_owned(),
        "denlib" => "mods/denlib".to_owned(),
        "coremods" => "coremods".to_owned(),
        "plugins" => "plugins".to_owned(),
        "texturepack" => "texturepacks".to_owned(),
        "resourcepack" => "resourcepacks".to_owned(),
        "shaderpack" => "shaderpacks".to_owned(),
        // The loader is installed from the pack's loader entry; jar mods are not supported.
        "root" | "forge" | "jar" | "mcpc" | "millenaire" => return Ok(None),
        other => return Err(Error::InvalidInput(format!("unknown mod type {other}"))),
    };
    Ok(Some(PathBuf::from(dir)))
}

fn sanitized(file: &str) -> String {
    file.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}
