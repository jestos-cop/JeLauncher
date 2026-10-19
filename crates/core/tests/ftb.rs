//! Talks to the real FTB API; run with `cargo test --test ftb -- --ignored --nocapture`.

use jelauncher_core::modpack::ModpackTarget;
use jelauncher_core::modplatform::{ModPlatforms, Provider, ResourceKind, SearchQuery, SortOrder};
use jelauncher_core::net::HttpClient;
use jelauncher_core::resources::list_resources;
use jelauncher_core::settings::NetworkSettings;

#[tokio::test]
#[ignore = "requires network access; downloads a large modpack"]
async fn installs_an_ftb_modpack() -> jelauncher_core::Result<()> {
    let platforms = ModPlatforms::new(HttpClient::new(&NetworkSettings::default())?, None);
    let page = platforms
        .search(&SearchQuery {
            provider: Provider::Ftb,
            text: "StoneBlock 3".into(),
            kind: ResourceKind::Modpack,
            game_version: None,
            loader: None,
            sort: SortOrder::Relevance,
            offset: 0,
            limit: 5,
        })
        .await?;
    let Some(pack) = page.hits.iter().find(|hit| hit.title == "FTB StoneBlock 3") else {
        panic!(
            "StoneBlock 3 not found in {:?}",
            page.hits.iter().map(|hit| &hit.title).collect::<Vec<_>>()
        );
    };
    let versions = platforms
        .versions(Provider::Ftb, &pack.id, ResourceKind::Modpack, None, None)
        .await?;
    let Some(latest) = versions.first() else {
        panic!("no versions");
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
    };
    let installed = platforms
        .install_modpack(Provider::Ftb, &pack.id, &latest.id, &target, |_| {})
        .await?;
    let instance = &installed.instance;
    let mods = list_resources(&instance.resource_dir(ResourceKind::Mod), ResourceKind::Mod)?;
    println!(
        "{} {} on {} with {:?}: {} mods",
        instance.config.name,
        latest.version_number,
        instance.config.minecraft_version,
        instance.config.loader,
        mods.len()
    );
    assert!(mods.len() > 50);
    assert!(instance.game_dir().join("config").is_dir());
    Ok(())
}
