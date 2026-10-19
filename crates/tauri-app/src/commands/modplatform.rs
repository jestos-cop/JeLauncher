use std::future::Future;
use std::path::Path;

use jelauncher_core::instance::Instance;

use jelauncher_core::modpack::{self, BlockedFile, ModpackTarget, PackInstall};
use jelauncher_core::modplatform::atlauncher::OptionalMod;
use jelauncher_core::modplatform::{
    InstallTarget, ProjectVersion, Provider, ResourceKind, ResourceUpdate, SearchPage, SearchQuery,
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
use tauri_specta::Event;
use tokio::sync::oneshot;

use crate::cancel::until_cancelled;
use crate::error::{CommandError, CommandResult};
use crate::events::ModpackProgressEvent;
use crate::state::AppState;

#[tauri::command]
#[specta::specta]
pub async fn search_resources(
    state: State<'_, AppState>,
    query: SearchQuery,
) -> CommandResult<SearchPage> {
    Ok(state.platforms.search(&query).await?)
}

/// Versions of a project that fit the instance's game version and loader.
#[tauri::command]
#[specta::specta]
pub async fn list_project_versions(
    state: State<'_, AppState>,
    instance_id: String,
    provider: Provider,
    project_id: String,
    kind: ResourceKind,
) -> CommandResult<Vec<ProjectVersion>> {
    let instance = Instance::open(&state.paths.instances(), &instance_id)?;
    let loader = instance.config.loader.as_ref().map(|loader| loader.kind);
    Ok(state
        .platforms
        .versions(
            provider,
            &project_id,
            kind,
            Some(&instance.config.minecraft_version),
            loader,
        )
        .await?)
}

/// Installs a version plus its required dependencies; returns the installed file names.
#[tauri::command]
#[specta::specta]
pub async fn install_project_version(
    state: State<'_, AppState>,
    instance_id: String,
    kind: ResourceKind,
    provider: Provider,
    project_id: String,
    version_id: String,
) -> CommandResult<Vec<String>> {
    let instance = Instance::open(&state.paths.instances(), &instance_id)?;
    let version = state
        .platforms
        .version(provider, &project_id, &version_id)
        .await?;
    let dir = instance.resource_dir(kind);
    Ok(state
        .platforms
        .install_with_dependencies(provider, version, &install_target(&instance, &dir, kind))
        .await?)
}

/// Newer Modrinth versions of the installed resources of `kind`.
#[tauri::command]
#[specta::specta]
pub async fn check_resource_updates(
    state: State<'_, AppState>,
    instance_id: String,
    kind: ResourceKind,
) -> CommandResult<Vec<ResourceUpdate>> {
    let instance = Instance::open(&state.paths.instances(), &instance_id)?;
    let dir = instance.resource_dir(kind);
    Ok(state
        .platforms
        .modrinth
        .check_updates(&install_target(&instance, &dir, kind))
        .await?)
}

/// Replaces an installed resource file with another version; returns the new file name.
#[tauri::command]
#[specta::specta]
pub async fn update_resource(
    state: State<'_, AppState>,
    instance_id: String,
    kind: ResourceKind,
    file_name: String,
    version_id: String,
) -> CommandResult<String> {
    let instance = Instance::open(&state.paths.instances(), &instance_id)?;
    let version = state.platforms.modrinth.version(&version_id).await?;
    Ok(state
        .platforms
        .modrinth
        .apply_update(&instance.resource_dir(kind), &file_name, &version)
        .await?)
}

fn install_target<'a>(
    instance: &'a Instance,
    dir: &'a Path,
    kind: ResourceKind,
) -> InstallTarget<'a> {
    InstallTarget {
        dir,
        kind,
        game_version: &instance.config.minecraft_version,
        loader: instance.config.loader.as_ref().map(|loader| loader.kind),
    }
}

/// A created instance and the pack files the user still has to download by hand.
#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct InstalledPack {
    pub instance_id: String,
    pub blocked: Vec<BlockedFile>,
}

impl From<PackInstall> for InstalledPack {
    fn from(install: PackInstall) -> Self {
        Self {
            instance_id: install.instance.id,
            blocked: install.blocked,
        }
    }
}

/// Choices the user made for a new pack instance.
#[derive(Debug, Clone, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PackOptions {
    /// `None` takes the pack's name.
    pub name: Option<String>,
    pub allow_untrusted_hosts: bool,
    /// Picked optional mods; `None` keeps the pack's defaults.
    pub optional_mods: Option<Vec<String>>,
}

/// Optional mods of an ATLauncher pack version.
#[tauri::command]
#[specta::specta]
pub async fn list_optional_mods(
    state: State<'_, AppState>,
    project_id: String,
    version_id: String,
) -> CommandResult<Vec<OptionalMod>> {
    Ok(state
        .platforms
        .atlauncher
        .pack_version(&project_id, &version_id)
        .await?
        .optional_mods())
}

/// The optional mods an ATLauncher share code selects.
#[tauri::command]
#[specta::specta]
pub async fn resolve_share_code(
    state: State<'_, AppState>,
    project_id: String,
    version_id: String,
    code: String,
) -> CommandResult<Vec<String>> {
    Ok(state
        .platforms
        .atlauncher
        .share_code(&project_id, &version_id, &code)
        .await?)
}

/// All published versions of a modpack, newest first.
#[tauri::command]
#[specta::specta]
pub async fn list_modpack_versions(
    state: State<'_, AppState>,
    provider: Provider,
    project_id: String,
) -> CommandResult<Vec<ProjectVersion>> {
    Ok(state
        .platforms
        .versions(provider, &project_id, ResourceKind::Modpack, None, None)
        .await?)
}

/// Installs a modpack version as a new instance.
#[tauri::command]
#[specta::specta]
pub async fn install_modpack(
    app: AppHandle,
    state: State<'_, AppState>,
    provider: Provider,
    project_id: String,
    version_id: String,
    options: PackOptions,
) -> CommandResult<InstalledPack> {
    let (instances_dir, download_dir) = (state.paths.instances(), state.paths.downloads_cache());
    let target = modpack_target(&state, &instances_dir, &download_dir, options)?;
    let install =
        state
            .platforms
            .install_modpack(provider, &project_id, &version_id, &target, |progress| {
                let _ = ModpackProgressEvent { progress }.emit(&app);
            });
    Ok(cancellable_modpack_task(&state, install).await?.into())
}

/// Imports an instance from a `.mrpack`, CurseForge pack or exported instance (path or URL).
#[tauri::command]
#[specta::specta]
pub async fn import_instance(
    app: AppHandle,
    state: State<'_, AppState>,
    source: String,
    options: PackOptions,
) -> CommandResult<InstalledPack> {
    let (instances_dir, download_dir) = (state.paths.instances(), state.paths.downloads_cache());
    let target = modpack_target(&state, &instances_dir, &download_dir, options)?;
    let import = modpack::import_instance(&state.platforms, source.trim(), &target, |progress| {
        let _ = ModpackProgressEvent { progress }.emit(&app);
    });
    Ok(cancellable_modpack_task(&state, import).await?.into())
}

/// Stops the modpack install or import in progress and removes the half-created instance.
#[tauri::command]
#[specta::specta]
pub fn cancel_modpack_install(state: State<'_, AppState>) -> CommandResult<()> {
    let cancel = state
        .modpack_cancel
        .lock()
        .map_err(|_| CommandError::poisoned())?
        .take();
    if let Some(cancel) = cancel {
        let _ = cancel.send(());
    }
    Ok(())
}

async fn cancellable_modpack_task<T>(
    state: &AppState,
    task: impl Future<Output = jelauncher_core::Result<T>>,
) -> CommandResult<T> {
    let (cancel_tx, mut cancel_rx) = oneshot::channel();
    *state
        .modpack_cancel
        .lock()
        .map_err(|_| CommandError::poisoned())? = Some(cancel_tx);
    until_cancelled(task, &mut cancel_rx).await
}

/// Switches a modpack instance to another version of its pack.
#[tauri::command]
#[specta::specta]
pub async fn update_modpack(
    app: AppHandle,
    state: State<'_, AppState>,
    instance_id: String,
    version_id: String,
) -> CommandResult<()> {
    if state
        .running
        .lock()
        .map_err(|_| CommandError::poisoned())?
        .contains_key(&instance_id)
    {
        return Err(CommandError::new("stop the instance before updating it"));
    }
    let mut instance = Instance::open(&state.paths.instances(), &instance_id)?;
    let concurrency = concurrent_downloads(&state)?;
    state
        .platforms
        .modrinth
        .update_modpack(
            &mut instance,
            &version_id,
            &state.paths.downloads_cache(),
            concurrency,
            |progress| {
                let _ = ModpackProgressEvent { progress }.emit(&app);
            },
        )
        .await?;
    Ok(())
}

fn modpack_target<'a>(
    state: &AppState,
    instances_dir: &'a Path,
    download_dir: &'a Path,
    options: PackOptions,
) -> CommandResult<ModpackTarget<'a>> {
    Ok(ModpackTarget {
        instances_dir,
        download_dir,
        name: options.name.filter(|name| !name.trim().is_empty()),
        concurrency: concurrent_downloads(state)?,
        allow_untrusted_hosts: options.allow_untrusted_hosts,
        optional_mods: options.optional_mods,
    })
}

fn concurrent_downloads(state: &AppState) -> CommandResult<usize> {
    Ok(state
        .settings
        .read()
        .map_err(|_| CommandError::poisoned())?
        .network
        .concurrent_downloads as usize)
}
