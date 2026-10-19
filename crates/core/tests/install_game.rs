//! Downloads a full game install. Set `JELAUNCHER_DATA_DIR` to reuse the files;
//! run with `cargo test --test install_game -- --ignored --nocapture`.

use std::path::PathBuf;

use jelauncher_core::meta::MetaService;
use jelauncher_core::minecraft::{Environment, Features, GameFiles};
use jelauncher_core::net::HttpClient;
use jelauncher_core::paths::DataPaths;
use jelauncher_core::settings::NetworkSettings;

#[tokio::test]
#[ignore = "downloads several hundred megabytes"]
async fn installs_latest_release() -> jelauncher_core::Result<()> {
    let root = std::env::var_os("JELAUNCHER_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("jelauncher-install-test"));
    let paths = DataPaths::new(root);
    let client = HttpClient::new(&NetworkSettings::default())?;
    let meta = MetaService::new(client.clone(), paths.meta());

    let latest = meta.version_manifest().await?.latest.release;
    let version = meta.version_json(&latest).await?;
    let files = GameFiles::resolve(&version, &Environment::current(Features::default()), &paths);
    let scratch = paths.root().join("install-test");
    files
        .install(
            &client,
            &scratch.join("natives"),
            &scratch.join("minecraft"),
            16,
            |p| {
                if p.completed % 500 == 0 || p.completed == p.total {
                    println!(
                        "{}/{} files, {} MB",
                        p.completed,
                        p.total,
                        p.downloaded_bytes / 1_000_000
                    );
                }
            },
        )
        .await?;

    assert!(files.client_jar.is_file());
    assert!(files.classpath.iter().all(|path| path.is_file()));
    println!("installed {latest}: {} libraries", files.classpath.len());
    Ok(())
}
