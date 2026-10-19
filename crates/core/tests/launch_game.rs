//! Starts real game instances with an offline account and watches their output.
//! Set `JELAUNCHER_DATA_DIR` to reuse downloaded files;
//! run with `cargo test --test launch_game -- --ignored --nocapture`.

use std::path::PathBuf;
use std::time::Duration;

use jelauncher_core::auth::offline_account;
use jelauncher_core::instance::{Instance, NewInstance, create_instance};
use jelauncher_core::launch::{LaunchOptions, LaunchProgress, Launcher, QuickPlay};
use jelauncher_core::meta::MetaService;
use jelauncher_core::minecraft::{LoaderKind, LoaderSpec};
use jelauncher_core::net::HttpClient;
use jelauncher_core::paths::DataPaths;
use jelauncher_core::settings::GlobalSettings;
use tokio::io::{AsyncBufReadExt, BufReader};

struct Harness {
    paths: DataPaths,
    settings: GlobalSettings,
    http: HttpClient,
    meta: MetaService,
    scratch: tempfile::TempDir,
}

impl Harness {
    fn new() -> jelauncher_core::Result<Self> {
        let root = std::env::var_os("JELAUNCHER_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::temp_dir().join("jelauncher-launch-test"));
        let paths = DataPaths::new(root);
        let settings = GlobalSettings::default();
        let http = HttpClient::new(&settings.network)?;
        let meta = MetaService::new(http.clone(), paths.meta());
        let scratch = tempfile::tempdir().map_err(|err| jelauncher_core::Error::Io {
            path: "tempdir".into(),
            source: err,
        })?;
        Ok(Self {
            paths,
            settings,
            http,
            meta,
            scratch,
        })
    }

    fn create(
        &self,
        minecraft_version: &str,
        loader: Option<LoaderSpec>,
    ) -> jelauncher_core::Result<Instance> {
        create_instance(
            self.scratch.path(),
            NewInstance {
                name: "Launch Test".into(),
                minecraft_version: minecraft_version.into(),
                loader,
                group: None,
                icon_key: None,
            },
        )
    }

    async fn launch(
        &self,
        instance: &Instance,
        seconds: u64,
        marker: &str,
    ) -> jelauncher_core::Result<bool> {
        self.launch_with(instance, &LaunchOptions::default(), seconds, marker)
            .await
    }

    /// Launches the instance, echoes its output for `seconds` and reports whether a line
    /// containing `marker` appeared while the game kept running.
    async fn launch_with(
        &self,
        instance: &Instance,
        options: &LaunchOptions,
        seconds: u64,
        marker: &str,
    ) -> jelauncher_core::Result<bool> {
        let launcher = Launcher {
            paths: &self.paths,
            meta: &self.meta,
            http: &self.http,
            settings: &self.settings,
        };
        let command = launcher
            .prepare(
                instance,
                &offline_account("JeTester")?,
                options,
                |progress| {
                    if !matches!(progress, LaunchProgress::Downloading(_)) {
                        println!("{progress:?}");
                    }
                },
            )
            .await?;
        println!("java: {}", command.java.display());

        let mut child = command.spawn()?;
        let (Some(stdout), Some(stderr)) = (child.stdout.take(), child.stderr.take()) else {
            panic!("output is not piped");
        };
        // An unread stderr pipe fills up and blocks the game.
        tokio::spawn(async move {
            let mut errors = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = errors.next_line().await {
                println!("! {line}");
            }
        });
        let mut lines = BufReader::new(stdout).lines();
        let mut saw_marker = false;
        let watch = async {
            while let Ok(Some(line)) = lines.next_line().await {
                saw_marker |= line.contains(marker);
                println!("| {line}");
            }
        };
        let _ = tokio::time::timeout(Duration::from_secs(seconds), watch).await;

        let still_running = child.try_wait().ok().flatten().is_none();
        let _ = child.kill().await;
        assert!(still_running, "game exited early");
        Ok(saw_marker)
    }

    async fn latest_loader(
        &self,
        kind: LoaderKind,
        minecraft_version: &str,
    ) -> jelauncher_core::Result<LoaderSpec> {
        let versions = self.meta.loader_versions(kind, minecraft_version).await?;
        let Some(version) = versions.iter().find(|v| v.stable).or(versions.first()) else {
            panic!("no {kind:?} for {minecraft_version}");
        };
        Ok(LoaderSpec {
            kind,
            version: version.version.clone(),
        })
    }
}

#[tokio::test]
#[ignore = "downloads the game and opens a window"]
async fn launches_latest_release() -> jelauncher_core::Result<()> {
    let harness = Harness::new()?;
    let latest = harness.meta.version_manifest().await?.latest.release;
    let instance = harness.create(&latest, None)?;
    harness.launch(&instance, 40, "").await?;
    Ok(())
}

#[tokio::test]
#[ignore = "downloads a modpack and opens a window"]
async fn launches_a_fabric_modpack() -> jelauncher_core::Result<()> {
    use jelauncher_core::modpack::ModpackTarget;
    use jelauncher_core::modplatform::ResourceKind;
    use jelauncher_core::modplatform::modrinth::Modrinth;

    let harness = Harness::new()?;
    let modrinth = Modrinth::new(harness.http.clone());
    let versions = modrinth
        .versions(
            "fabulously-optimized",
            ResourceKind::Modpack,
            Some("1.21.4"),
            Some(LoaderKind::Fabric),
        )
        .await?;
    let Some(pack) = versions.first() else {
        panic!("no Fabulously Optimized for 1.21.4");
    };
    let instance = modrinth
        .install_modpack(
            &pack.id,
            &ModpackTarget {
                instances_dir: harness.scratch.path(),
                download_dir: &harness.paths.downloads_cache(),
                name: None,
                concurrency: 8,
            },
            |_| {},
        )
        .await?;
    let loaded = harness.launch(&instance, 60, "Loading").await?;
    assert!(loaded, "Fabric did not report loading mods");
    Ok(())
}

#[tokio::test]
#[ignore = "runs the NeoForge installer and opens a window"]
async fn launches_neoforge() -> jelauncher_core::Result<()> {
    let harness = Harness::new()?;
    let loader = harness
        .latest_loader(LoaderKind::NeoForge, "1.21.4")
        .await?;
    println!("NeoForge {}", loader.version);
    let instance = harness.create("1.21.4", Some(loader))?;
    let loaded = harness.launch(&instance, 90, "ModLauncher").await?;
    assert!(loaded, "NeoForge did not start ModLauncher");
    Ok(())
}

#[tokio::test]
#[ignore = "runs the Forge installer and opens a window"]
async fn launches_forge() -> jelauncher_core::Result<()> {
    let harness = Harness::new()?;
    let loader = harness.latest_loader(LoaderKind::Forge, "1.21.4").await?;
    println!("Forge {}", loader.version);
    let instance = harness.create("1.21.4", Some(loader))?;
    let loaded = harness.launch(&instance, 90, "Forge").await?;
    assert!(loaded, "Forge did not report starting");
    Ok(())
}

#[tokio::test]
#[ignore = "downloads the game and opens a window"]
async fn quick_play_joins_a_server() -> jelauncher_core::Result<()> {
    let io = |err| jelauncher_core::Error::Io {
        path: "listener".into(),
        source: err,
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(io)?;
    let port = listener.local_addr().map_err(io)?.port();
    let connection = tokio::spawn(async move { listener.accept().await.is_ok() });

    let harness = Harness::new()?;
    let instance = harness.create("1.21.4", None)?;
    let options = LaunchOptions {
        quick_play: Some(QuickPlay::Server(format!("127.0.0.1:{port}"))),
        demo: false,
    };
    harness.launch_with(&instance, &options, 90, "").await?;
    let connected = connection.is_finished() && connection.await.unwrap_or(false);
    assert!(connected, "the game did not connect to the server");
    Ok(())
}
