//! Records and recovers resource sources against the real platforms; needs `CURSEFORGE_API_KEY`.
//! Run with `cargo test --test sources -- --ignored --nocapture`.

use jelauncher_core::minecraft::LoaderKind;
use jelauncher_core::modpack::ModpackTarget;
use jelauncher_core::modplatform::{
    InstallTarget, ModPlatforms, PlatformConfig, Provider, ResourceKind,
};
use jelauncher_core::net::HttpClient;
use jelauncher_core::resources::{ResourceSource, read_sources, remove_source};
use jelauncher_core::settings::{NetworkSettings, SourceLookupOrder};

fn platforms() -> jelauncher_core::Result<ModPlatforms> {
    Ok(ModPlatforms::new(
        HttpClient::new(&NetworkSettings::default())?,
        PlatformConfig {
            curseforge_api_key: std::env::var("CURSEFORGE_API_KEY").ok(),
            ..Default::default()
        },
    ))
}

fn tempdir() -> jelauncher_core::Result<tempfile::TempDir> {
    tempfile::tempdir().map_err(|err| jelauncher_core::Error::Io {
        path: "tempdir".into(),
        source: err,
    })
}

fn sources(dir: &std::path::Path) -> Vec<ResourceSource> {
    read_sources(dir, ResourceKind::Mod)
        .into_iter()
        .map(|(_, source)| source)
        .collect()
}

#[tokio::test]
#[ignore = "requires network access and an API key"]
async fn records_installs_and_identifies_unknown_files() -> jelauncher_core::Result<()> {
    let platforms = platforms()?;
    let dir = tempdir()?;
    let target = InstallTarget {
        dir: dir.path(),
        kind: ResourceKind::Mod,
        game_version: "1.21.4",
        loader: Some(LoaderKind::Fabric),
        track_sources: true,
    };
    let versions = platforms
        .versions(
            Provider::Modrinth,
            "iris",
            ResourceKind::Mod,
            Some("1.21.4"),
            Some(LoaderKind::Fabric),
        )
        .await?;
    let Some(iris) = versions.into_iter().next() else {
        panic!("no Iris for 1.21.4");
    };
    let installed = platforms
        .install_with_dependencies(Provider::Modrinth, iris, &target)
        .await?;
    let recorded = sources(dir.path());
    println!(
        "installed {installed:?}, recorded {:?}",
        recorded
            .iter()
            .map(|s| (&s.slug, &s.file_name))
            .collect::<Vec<_>>()
    );
    assert_eq!(recorded.len(), installed.len());
    assert!(
        recorded
            .iter()
            .all(|source| source.provider == Provider::Modrinth)
    );

    // Forget them, then find them again by hash.
    for name in &installed {
        remove_source(dir.path(), ResourceKind::Mod, name)?;
    }
    let report = platforms
        .identify_sources(dir.path(), ResourceKind::Mod, false, Default::default())
        .await?;
    println!("identify: {report:?}");
    assert_eq!(report.identified as usize, installed.len());
    assert_eq!(sources(dir.path()).len(), installed.len());

    // A CurseForge-only file is found by fingerprint.
    let curseforge = platforms
        .versions(
            Provider::CurseForge,
            "238222",
            ResourceKind::Mod,
            Some("1.21.1"),
            Some(LoaderKind::NeoForge),
        )
        .await?;
    let Some(jei) = curseforge.into_iter().next() else {
        panic!("no JEI for 1.21.1");
    };
    let jei_file =
        jelauncher_core::modplatform::download_version(platforms.http(), &jei, dir.path()).await?;
    let disabled = format!("{jei_file}.disabled");
    std::fs::rename(dir.path().join(&jei_file), dir.path().join(&disabled)).map_err(|err| {
        jelauncher_core::Error::Io {
            path: disabled.clone().into(),
            source: err,
        }
    })?;
    let report = platforms
        .identify_sources(dir.path(), ResourceKind::Mod, false, Default::default())
        .await?;
    println!("identify after JEI: {report:?}");
    let jei_source = sources(dir.path())
        .into_iter()
        .find(|source| source.file_name == jei_file);
    println!(
        "JEI source: {:?}",
        jei_source.as_ref().map(|source| source.provider)
    );
    assert!(
        jei_source.is_some(),
        "a disabled file is identified under its enabled name"
    );

    // Asking CurseForge first finds Iris there, although Modrinth has it too.
    for name in &installed {
        remove_source(dir.path(), ResourceKind::Mod, name)?;
    }
    platforms
        .identify_sources(
            dir.path(),
            ResourceKind::Mod,
            false,
            SourceLookupOrder::CurseForgeFirst,
        )
        .await?;
    let iris_source = sources(dir.path())
        .into_iter()
        .find(|source| source.file_name == installed[0]);
    println!(
        "Iris source: {:?}",
        iris_source.as_ref().map(|s| s.provider)
    );
    assert_eq!(
        iris_source.map(|source| source.provider),
        Some(Provider::CurseForge)
    );
    Ok(())
}

#[tokio::test]
#[ignore = "requires network access; downloads a modpack"]
async fn modpack_installs_record_locked_sources() -> jelauncher_core::Result<()> {
    let platforms = platforms()?;
    let versions = platforms
        .versions(
            Provider::Modrinth,
            "fabulously-optimized",
            ResourceKind::Modpack,
            Some("1.21.4"),
            Some(LoaderKind::Fabric),
        )
        .await?;
    let Some(pack) = versions.first() else {
        panic!("no pack");
    };
    let dir = tempdir()?;
    let target = ModpackTarget {
        instances_dir: &dir.path().join("instances"),
        download_dir: &dir.path().join("downloads"),
        name: None,
        concurrency: 8,
        allow_untrusted_hosts: false,
        optional_mods: None,
        track_sources: true,
        source_order: Default::default(),
    };
    let installed = platforms
        .install_modpack(
            Provider::Modrinth,
            "fabulously-optimized",
            &pack.id,
            &target,
            |_| {},
        )
        .await?;
    let mods_dir = installed.instance.resource_dir(ResourceKind::Mod);
    let recorded = sources(&mods_dir);
    let jars = std::fs::read_dir(&mods_dir)
        .map_err(|err| jelauncher_core::Error::Io {
            path: mods_dir.clone(),
            source: err,
        })?
        .flatten()
        .filter(|entry| entry.file_name().to_string_lossy().ends_with(".jar"))
        .count();
    println!("{} of {jars} mods have a source", recorded.len());
    assert!(
        recorded.len() * 10 >= jars * 9,
        "most pack mods should be identified"
    );
    assert!(recorded.iter().all(|source| source.lock_update));
    Ok(())
}

#[tokio::test]
#[ignore = "requires network access and an API key"]
async fn updates_curseforge_files_on_curseforge() -> jelauncher_core::Result<()> {
    let platforms = platforms()?;
    let dir = tempdir()?;
    let target = InstallTarget {
        dir: dir.path(),
        kind: ResourceKind::Mod,
        game_version: "1.21.1",
        loader: Some(LoaderKind::NeoForge),
        track_sources: true,
    };
    let mut versions = platforms
        .versions(
            Provider::CurseForge,
            "238222",
            ResourceKind::Mod,
            Some("1.21.1"),
            Some(LoaderKind::NeoForge),
        )
        .await?;
    versions.sort_by(|a, b| a.published.cmp(&b.published));
    let Some(oldest) = versions.first().cloned() else {
        panic!("no JEI for 1.21.1");
    };
    let old_name = platforms
        .install_with_dependencies(Provider::CurseForge, oldest.clone(), &target)
        .await?
        .remove(0);
    let channel = jelauncher_core::settings::UpdateChannel::Alpha;
    let updates = platforms.check_updates(&target, channel).await?;
    let Some(update) = updates.iter().find(|update| update.file_name == old_name) else {
        panic!("{old_name} has no update in {updates:?}");
    };
    println!(
        "{old_name} -> {} on {:?}",
        update.version.version_number, update.provider
    );
    assert_eq!(update.provider, Provider::CurseForge);

    // Locked files are left alone.
    jelauncher_core::resources::set_update_lock(dir.path(), ResourceKind::Mod, &old_name, true)?;
    assert!(
        platforms
            .check_updates(&target, channel)
            .await?
            .iter()
            .all(|u| u.file_name != old_name)
    );
    jelauncher_core::resources::set_update_lock(dir.path(), ResourceKind::Mod, &old_name, false)?;

    let new_name = platforms
        .update_resource(
            &target,
            &old_name,
            update.provider,
            &update.version.project_id,
            &update.version.id,
        )
        .await?;
    let source = sources(dir.path())
        .into_iter()
        .find(|source| source.file_name == new_name);
    assert_eq!(
        source.map(|source| source.version_id),
        Some(update.version.id.clone())
    );
    assert!(!dir.path().join(&old_name).exists() || old_name == new_name);
    Ok(())
}
