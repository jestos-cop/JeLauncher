//! Hits the real Mojang endpoints; run with `cargo test -- --ignored`.

use jelauncher_core::meta::MetaService;
use jelauncher_core::minecraft::{Environment, Features};
use jelauncher_core::net::HttpClient;
use jelauncher_core::settings::NetworkSettings;

#[tokio::test]
#[ignore = "requires network access"]
async fn parses_versions_from_every_era() -> jelauncher_core::Result<()> {
    let cache = tempfile::tempdir().map_err(|err| jelauncher_core::Error::Io {
        path: "tempdir".into(),
        source: err,
    })?;
    let meta = MetaService::new(HttpClient::new(&NetworkSettings::default())?, cache.path());
    let manifest = meta.version_manifest().await?;
    let env = Environment::current(Features::default());

    for id in [
        manifest.latest.release.as_str(),
        "1.21.4",
        "1.12.2",
        "1.8.9",
        "b1.7.3",
    ] {
        let version = meta.version_json(id).await?;
        let classpath = version
            .libraries
            .iter()
            .filter(|l| env.allows(&l.rules))
            .count();
        assert!(classpath > 0, "{id} has no libraries for this platform");
        assert!(version.arguments.is_some() || version.minecraft_arguments.is_some());
    }
    Ok(())
}
