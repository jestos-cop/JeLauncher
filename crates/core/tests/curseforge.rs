//! Talks to the real CurseForge API; needs `CURSEFORGE_API_KEY`.
//! Run with `cargo test --test curseforge -- --ignored --nocapture`.

use jelauncher_core::minecraft::LoaderKind;
use jelauncher_core::modplatform::{
    InstallTarget, ModPlatforms, PlatformConfig, Provider, ResourceKind, SearchQuery, SortOrder,
};
use jelauncher_core::net::HttpClient;
use jelauncher_core::resources::list_resources;
use jelauncher_core::settings::NetworkSettings;

#[tokio::test]
#[ignore = "requires network access and an API key"]
async fn searches_and_installs_a_mod_with_dependencies() -> jelauncher_core::Result<()> {
    let platforms = ModPlatforms::new(
        HttpClient::new(&NetworkSettings::default())?,
        PlatformConfig {
            curseforge_api_key: std::env::var("CURSEFORGE_API_KEY").ok(),
            ..Default::default()
        },
    );
    let (game_version, loader) = ("1.21.1", LoaderKind::NeoForge);
    let page = platforms
        .search(&SearchQuery {
            provider: Provider::CurseForge,
            text: "jei".into(),
            kind: ResourceKind::Mod,
            game_version: Some(game_version.into()),
            loader: Some(loader),
            sort: SortOrder::Downloads,
            offset: 0,
            limit: 5,
        })
        .await?;
    println!("{} hits", page.total);
    let Some(jei) = page.hits.iter().find(|hit| hit.slug == "jei") else {
        panic!("JEI not found");
    };

    let versions = platforms
        .versions(
            Provider::CurseForge,
            &jei.id,
            ResourceKind::Mod,
            Some(game_version),
            Some(loader),
        )
        .await?;
    let Some(latest) = versions.into_iter().next() else {
        panic!("no JEI for {game_version}");
    };
    let dir = tempfile::tempdir().map_err(|err| jelauncher_core::Error::Io {
        path: "tempdir".into(),
        source: err,
    })?;
    let target = InstallTarget {
        dir: dir.path(),
        kind: ResourceKind::Mod,
        game_version,
        loader: Some(loader),
        track_sources: false,
    };
    let installed = platforms
        .install_with_dependencies(Provider::CurseForge, latest, &target)
        .await?;
    println!("installed {installed:?}");
    let mods = list_resources(dir.path(), ResourceKind::Mod)?;
    assert_eq!(mods.len(), installed.len());
    assert!(installed[0].starts_with("jei-"));
    Ok(())
}

#[tokio::test]
#[ignore = "requires network access and an API key; downloads a modpack"]
async fn installs_a_curseforge_modpack() -> jelauncher_core::Result<()> {
    use jelauncher_core::modpack::ModpackTarget;

    let platforms = ModPlatforms::new(
        HttpClient::new(&NetworkSettings::default())?,
        PlatformConfig {
            curseforge_api_key: std::env::var("CURSEFORGE_API_KEY").ok(),
            modrinth_fallback_for_blocked: true,
            ..Default::default()
        },
    );
    let page = platforms
        .search(&SearchQuery {
            provider: Provider::CurseForge,
            text: "Fabulously Optimized".into(),
            kind: ResourceKind::Modpack,
            game_version: Some("1.21.4".into()),
            loader: Some(LoaderKind::Fabric),
            sort: SortOrder::Downloads,
            offset: 0,
            limit: 5,
        })
        .await?;
    let Some(pack) = page
        .hits
        .iter()
        .find(|hit| hit.slug == "fabulously-optimized")
    else {
        panic!("Fabulously Optimized not found");
    };
    let versions = platforms
        .versions(
            Provider::CurseForge,
            &pack.id,
            ResourceKind::Modpack,
            Some("1.21.4"),
            Some(LoaderKind::Fabric),
        )
        .await?;
    let Some(latest) = versions.first() else {
        panic!("no pack version for 1.21.4");
    };
    let dir = tempfile::tempdir().map_err(|err| jelauncher_core::Error::Io {
        path: "tempdir".into(),
        source: err,
    })?;
    let target = ModpackTarget {
        instances_dir: &dir.path().join("instances"),
        download_dir: &dir.path().join("downloads"),
        name: None,
        concurrency: 8,
        allow_untrusted_hosts: false,
        optional_mods: None,
        track_sources: false,
        source_order: Default::default(),
    };
    let installed = platforms
        .install_modpack(Provider::CurseForge, &pack.id, &latest.id, &target, |_| {})
        .await?;
    let instance = &installed.instance;
    let mods = list_resources(&instance.resource_dir(ResourceKind::Mod), ResourceKind::Mod)?;
    println!(
        "{} {} with {:?}: {} mods, {} blocked: {:?}",
        instance.config.name,
        instance.config.minecraft_version,
        instance.config.loader,
        mods.len(),
        installed.blocked.len(),
        installed
            .blocked
            .iter()
            .map(|file| &file.file_name)
            .collect::<Vec<_>>()
    );
    assert!(mods.len() + installed.blocked.len() > 10);
    assert!(instance.config.managed_pack.is_some());
    Ok(())
}

#[tokio::test]
#[ignore = "requires network access and an API key"]
async fn exports_a_pack_that_references_curseforge_files() -> jelauncher_core::Result<()> {
    use jelauncher_core::archive::{entry_names, read_entry_to_string};
    use jelauncher_core::instance::{NewInstance, create_instance};
    use jelauncher_core::minecraft::LoaderSpec;
    use jelauncher_core::modpack::{CurseForgeExport, CurseManifest, export_curseforge_pack};

    let platforms = ModPlatforms::new(
        HttpClient::new(&NetworkSettings::default())?,
        PlatformConfig {
            curseforge_api_key: std::env::var("CURSEFORGE_API_KEY").ok(),
            ..Default::default()
        },
    );
    let io = |err| jelauncher_core::Error::Io {
        path: "tempdir".into(),
        source: err,
    };
    let dir = tempfile::tempdir().map_err(io)?;
    let instance = create_instance(
        dir.path(),
        NewInstance {
            name: "Export".into(),
            minecraft_version: "1.21.1".into(),
            loader: Some(LoaderSpec {
                kind: LoaderKind::NeoForge,
                version: "21.1.77".into(),
            }),
            group: None,
            icon_key: None,
        },
    )?;
    let mods_dir = instance.resource_dir(ResourceKind::Mod);
    let versions = platforms
        .versions(
            Provider::CurseForge,
            "238222",
            ResourceKind::Mod,
            Some("1.21.1"),
            Some(LoaderKind::NeoForge),
        )
        .await?;
    let Some(jei) = versions.into_iter().next() else {
        panic!("no JEI for 1.21.1");
    };
    let target = InstallTarget {
        dir: &mods_dir,
        kind: ResourceKind::Mod,
        game_version: "1.21.1",
        loader: Some(LoaderKind::NeoForge),
        track_sources: false,
    };
    let installed = platforms
        .install_with_dependencies(Provider::CurseForge, jei, &target)
        .await?;
    std::fs::write(mods_dir.join("homemade.jar"), b"not on curseforge").map_err(io)?;

    let output = dir.path().join("pack.zip");
    export_curseforge_pack(
        &platforms.curseforge,
        &instance,
        &CurseForgeExport {
            output: &output,
            version: "1.0".into(),
            author: "JeTester".into(),
            blocked: &[],
        },
    )
    .await?;
    let manifest: CurseManifest =
        serde_json::from_str(&read_entry_to_string(&output, "manifest.json")?)
            .unwrap_or_else(|err| panic!("{err}"));
    println!("{} files in the manifest", manifest.files.len());
    assert_eq!(manifest.files.len(), installed.len());
    assert!(entry_names(&output)?.contains(&"overrides/mods/homemade.jar".to_owned()));
    Ok(())
}
