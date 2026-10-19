//! Talks to the real CurseForge API; needs `CURSEFORGE_API_KEY`.
//! Run with `cargo test --test curseforge -- --ignored --nocapture`.

use jelauncher_core::minecraft::LoaderKind;
use jelauncher_core::modplatform::{
    InstallTarget, ModPlatforms, Provider, ResourceKind, SearchQuery, SortOrder,
};
use jelauncher_core::net::HttpClient;
use jelauncher_core::resources::list_resources;
use jelauncher_core::settings::NetworkSettings;

#[tokio::test]
#[ignore = "requires network access and an API key"]
async fn searches_and_installs_a_mod_with_dependencies() -> jelauncher_core::Result<()> {
    let platforms = ModPlatforms::new(
        HttpClient::new(&NetworkSettings::default())?,
        std::env::var("CURSEFORGE_API_KEY").ok(),
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
