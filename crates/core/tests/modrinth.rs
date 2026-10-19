//! Talks to the real Modrinth API; run with `cargo test --test modrinth -- --ignored --nocapture`.

use jelauncher_core::minecraft::LoaderKind;
use jelauncher_core::modplatform::modrinth::Modrinth;
use jelauncher_core::modplatform::{ResourceKind, SearchQuery, SortOrder};
use jelauncher_core::net::HttpClient;
use jelauncher_core::resources::list_resources;
use jelauncher_core::settings::NetworkSettings;

#[tokio::test]
#[ignore = "requires network access"]
async fn searches_and_installs_a_mod() -> jelauncher_core::Result<()> {
    let modrinth = Modrinth::new(HttpClient::new(&NetworkSettings::default())?);
    let game_version = "1.21.4";
    let page = modrinth
        .search(&SearchQuery {
            text: "sodium".into(),
            kind: ResourceKind::Mod,
            game_version: Some(game_version.into()),
            loader: Some(LoaderKind::Fabric),
            sort: SortOrder::Relevance,
            offset: 0,
            limit: 5,
        })
        .await?;
    let Some(sodium) = page.hits.iter().find(|hit| hit.slug == "sodium") else {
        panic!("sodium not found in {} hits", page.total);
    };

    let versions = modrinth
        .versions(
            &sodium.id,
            ResourceKind::Mod,
            Some(game_version),
            Some(LoaderKind::Fabric),
        )
        .await?;
    let Some(latest) = versions.first() else {
        panic!("no compatible sodium versions");
    };

    let dir = tempfile::tempdir().map_err(|err| jelauncher_core::Error::Io {
        path: "tempdir".into(),
        source: err,
    })?;
    let file_name = modrinth.install(latest, dir.path()).await?;
    let mods = list_resources(dir.path(), ResourceKind::Mod)?;
    println!("installed {file_name}: {:?}", mods[0].metadata);
    assert_eq!(
        mods[0].metadata.as_ref().map(|m| m.id.as_str()),
        Some("sodium")
    );
    Ok(())
}

#[tokio::test]
#[ignore = "requires network access"]
async fn installs_required_dependencies() -> jelauncher_core::Result<()> {
    use jelauncher_core::modplatform::modrinth::InstallTarget;

    let modrinth = Modrinth::new(HttpClient::new(&NetworkSettings::default())?);
    let game_version = "1.21.4";
    let versions = modrinth
        .versions(
            "iris",
            ResourceKind::Mod,
            Some(game_version),
            Some(LoaderKind::Fabric),
        )
        .await?;
    let Some(iris) = versions.into_iter().next() else {
        panic!("no compatible iris versions");
    };
    let dir = tempfile::tempdir().map_err(|err| jelauncher_core::Error::Io {
        path: "tempdir".into(),
        source: err,
    })?;
    let target = InstallTarget {
        dir: dir.path(),
        kind: ResourceKind::Mod,
        game_version,
        loader: Some(LoaderKind::Fabric),
    };
    let installed = modrinth.install_with_dependencies(iris, &target).await?;
    println!("installed {installed:?}");
    let ids: Vec<_> = list_resources(dir.path(), ResourceKind::Mod)?
        .into_iter()
        .filter_map(|file| file.metadata.map(|m| m.id))
        .collect();
    assert!(
        ids.contains(&"iris".to_owned()) && ids.contains(&"sodium".to_owned()),
        "{ids:?}"
    );
    Ok(())
}
