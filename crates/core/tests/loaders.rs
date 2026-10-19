//! Prepares Fabric and Quilt instances against the real metadata servers without starting
//! the game. Set `JELAUNCHER_DATA_DIR` to reuse downloads;
//! run with `cargo test --test loaders -- --ignored --nocapture`.

use std::path::PathBuf;

use jelauncher_core::auth::offline_account;
use jelauncher_core::instance::{NewInstance, create_instance};
use jelauncher_core::launch::Launcher;
use jelauncher_core::meta::MetaService;
use jelauncher_core::minecraft::{LoaderKind, LoaderSpec};
use jelauncher_core::net::HttpClient;
use jelauncher_core::paths::DataPaths;
use jelauncher_core::settings::GlobalSettings;

#[tokio::test]
#[ignore = "downloads game files"]
async fn prepares_fabric_and_quilt() -> jelauncher_core::Result<()> {
    let root = std::env::var_os("JELAUNCHER_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("jelauncher-loader-test"));
    let paths = DataPaths::new(root);
    let settings = GlobalSettings::default();
    let http = HttpClient::new(&settings.network)?;
    let meta = MetaService::new(http.clone(), paths.meta());
    let game_version = meta.version_manifest().await?.latest.release;
    let scratch = tempfile::tempdir().map_err(|err| jelauncher_core::Error::Io {
        path: "tempdir".into(),
        source: err,
    })?;
    let launcher = Launcher {
        paths: &paths,
        meta: &meta,
        http: &http,
        settings: &settings,
    };

    for (kind, main_class) in [
        (
            LoaderKind::Fabric,
            "net.fabricmc.loader.impl.launch.knot.KnotClient",
        ),
        (
            LoaderKind::Quilt,
            "org.quiltmc.loader.impl.launch.knot.KnotClient",
        ),
    ] {
        let versions = meta.loader_versions(kind, &game_version).await?;
        let Some(loader) = versions.iter().find(|v| v.stable) else {
            panic!("no stable {kind:?} loader for {game_version}");
        };
        let instance = create_instance(
            scratch.path(),
            NewInstance {
                name: format!("{kind:?}"),
                minecraft_version: game_version.clone(),
                loader: Some(LoaderSpec {
                    kind,
                    version: loader.version.clone(),
                }),
                group: None,
                icon_key: None,
            },
        )?;
        let command = launcher
            .prepare(
                &instance,
                &offline_account("JeTester")?,
                &Default::default(),
                |_| {},
            )
            .await?;
        println!("{kind:?} {} on {game_version}", loader.version);
        assert!(command.args.iter().any(|arg| arg == main_class));
    }
    Ok(())
}
