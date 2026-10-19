//! Starts the latest release with an offline account and watches its output.
//! Set `JELAUNCHER_DATA_DIR` to reuse downloaded files;
//! run with `cargo test --test launch_game -- --ignored --nocapture`.

use std::path::PathBuf;
use std::time::Duration;

use jelauncher_core::auth::offline_account;
use jelauncher_core::instance::{NewInstance, create_instance};
use jelauncher_core::launch::{LaunchProgress, Launcher};
use jelauncher_core::meta::MetaService;
use jelauncher_core::net::HttpClient;
use jelauncher_core::paths::DataPaths;
use jelauncher_core::settings::GlobalSettings;
use tokio::io::{AsyncBufReadExt, BufReader};

#[tokio::test]
#[ignore = "downloads the game and opens a window"]
async fn launches_latest_release() -> jelauncher_core::Result<()> {
    let root = std::env::var_os("JELAUNCHER_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("jelauncher-launch-test"));
    let paths = DataPaths::new(root);
    let settings = GlobalSettings::default();
    let http = HttpClient::new(&settings.network)?;
    let meta = MetaService::new(http.clone(), paths.meta());
    let latest = meta.version_manifest().await?.latest.release;

    let scratch = tempfile::tempdir().map_err(|err| jelauncher_core::Error::Io {
        path: "tempdir".into(),
        source: err,
    })?;
    let instance = create_instance(
        scratch.path(),
        NewInstance {
            name: "Launch Test".into(),
            minecraft_version: latest,
            loader: None,
            group: None,
            icon_key: None,
        },
    )?;
    let launcher = Launcher {
        paths: &paths,
        meta: &meta,
        http: &http,
        settings: &settings,
    };
    let command = launcher
        .prepare(&instance, &offline_account("JeTester")?, |progress| {
            if !matches!(progress, LaunchProgress::Downloading(_)) {
                println!("{progress:?}");
            }
        })
        .await?;
    println!("java: {}", command.java.display());

    let mut child = command.spawn()?;
    let Some(stdout) = child.stdout.take() else {
        panic!("stdout is not piped");
    };
    let mut lines = BufReader::new(stdout).lines();
    let watch = async {
        while let Ok(Some(line)) = lines.next_line().await {
            println!("| {line}");
        }
    };
    let _ = tokio::time::timeout(Duration::from_secs(40), watch).await;

    let still_running = child.try_wait().ok().flatten().is_none();
    let _ = child.kill().await;
    assert!(still_running, "game exited early");
    Ok(())
}
