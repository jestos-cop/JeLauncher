//! Talks to the real ATLauncher servers; run with `cargo test --test atlauncher -- --ignored --nocapture`.

use jelauncher_core::modpack::ModpackTarget;
use jelauncher_core::modplatform::{ModPlatforms, Provider, ResourceKind, SearchQuery, SortOrder};
use jelauncher_core::net::HttpClient;
use jelauncher_core::resources::list_resources;
use jelauncher_core::settings::NetworkSettings;

#[tokio::test]
#[ignore = "requires network access; downloads a modpack"]
async fn installs_a_pack_with_optional_mods_and_extracted_files() -> jelauncher_core::Result<()> {
    let platforms = ModPlatforms::new(
        HttpClient::new(&NetworkSettings::default())?,
        Default::default(),
    );
    let page = platforms
        .search(&SearchQuery {
            provider: Provider::AtLauncher,
            text: "film".into(),
            kind: ResourceKind::Modpack,
            game_version: None,
            loader: None,
            sort: SortOrder::Relevance,
            offset: 0,
            limit: 10,
        })
        .await?;
    let Some(pack) = page.hits.iter().find(|hit| hit.id == "FilmPack") else {
        panic!(
            "FilmPack not found in {:?}",
            page.hits.iter().map(|hit| &hit.id).collect::<Vec<_>>()
        );
    };
    let versions = platforms
        .versions(
            Provider::AtLauncher,
            &pack.id,
            ResourceKind::Modpack,
            None,
            None,
        )
        .await?;
    let Some(latest) = versions.first() else {
        panic!("no versions");
    };
    let manifest = platforms
        .atlauncher
        .pack_version(&pack.id, &latest.id)
        .await?;
    let optional = manifest.optional_mods();
    println!("optional: {optional:?}");

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
        optional_mods: Some(optional.iter().map(|module| module.name.clone()).collect()),
        track_sources: false,
        source_order: Default::default(),
    };
    let installed = platforms
        .install_modpack(Provider::AtLauncher, &pack.id, &latest.id, &target, |_| {})
        .await?;
    let instance = &installed.instance;
    let mods = list_resources(&instance.resource_dir(ResourceKind::Mod), ResourceKind::Mod)?;
    println!(
        "{} {} on {} with {:?}: {} mods, {} blocked",
        instance.config.name,
        latest.version_number,
        instance.config.minecraft_version,
        instance.config.loader,
        mods.len(),
        installed.blocked.len()
    );
    assert!(mods.len() > 10);
    assert!(instance.game_dir().join("config").is_dir());
    Ok(())
}
