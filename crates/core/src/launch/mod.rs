mod args;

use std::path::PathBuf;
use std::process::Stdio;

use serde::Serialize;
use tokio::process::{Child, Command};

pub use args::{ArgsInput, LAUNCHER_NAME, build_arguments};

use crate::auth::Account;
use crate::instance::Instance;
use crate::java::{self, check_java};
use crate::meta::MetaService;
use crate::minecraft::{Environment, Features, GameFiles, JavaVersion};
use crate::net::{DownloadProgress, HttpClient};
use crate::paths::DataPaths;
use crate::settings::GlobalSettings;
use crate::{Error, Result};

/// Java versions older than 1.17 do not declare one and run on Java 8.
const DEFAULT_JAVA_MAJOR: u32 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, specta::Type)]
#[serde(tag = "stage", rename_all = "camelCase")]
pub enum LaunchProgress {
    Preparing,
    Downloading(DownloadProgress),
    Starting,
}

pub struct Launcher<'a> {
    pub paths: &'a DataPaths,
    pub meta: &'a MetaService,
    pub http: &'a HttpClient,
    pub settings: &'a GlobalSettings,
}

#[derive(Debug, Clone)]
pub struct LaunchCommand {
    pub java: PathBuf,
    pub args: Vec<String>,
    pub working_dir: PathBuf,
}

impl Launcher<'_> {
    /// Makes sure every game file is present and builds the command line.
    pub async fn prepare(
        &self,
        instance: &Instance,
        account: &Account,
        on_progress: impl Fn(LaunchProgress),
    ) -> Result<LaunchCommand> {
        on_progress(LaunchProgress::Preparing);
        let game_version = &instance.config.minecraft_version;
        let mut version = self.meta.version_json(game_version).await?;
        if let Some(loader) = &instance.config.loader {
            version.apply_loader(self.meta.loader_profile(loader, game_version).await?);
        }
        let settings = instance.config.settings.resolve(self.settings);
        let env = Environment::current(Features {
            custom_resolution: !settings.window_maximized,
            ..Features::default()
        });

        let files = GameFiles::resolve(&version, &env, self.paths);
        let game_dir = instance.game_dir();
        files
            .install(
                self.http,
                &instance.natives_dir(),
                &game_dir,
                self.settings.network.concurrent_downloads as usize,
                |progress| on_progress(LaunchProgress::Downloading(progress)),
            )
            .await?;

        let java = self
            .find_java(
                settings.java_path.as_deref(),
                version.java_version.as_ref(),
                &on_progress,
            )
            .await?;

        let mut classpath = files.classpath.clone();
        classpath.push(files.client_jar.clone());
        let index = files.read_asset_index()?;
        let args = build_arguments(&ArgsInput {
            version: &version,
            env: &env,
            settings: &settings,
            account,
            classpath: &classpath,
            game_dir: &game_dir,
            natives_dir: &instance.natives_dir(),
            assets_dir: files.assets_dir(),
            legacy_assets_dir: &index.legacy_dir(
                files.assets_dir(),
                files.asset_index_id(),
                &game_dir,
            ),
            libraries_dir: &self.paths.libraries(),
        });
        on_progress(LaunchProgress::Starting);
        Ok(LaunchCommand {
            java,
            args,
            working_dir: game_dir,
        })
    }

    async fn find_java(
        &self,
        configured: Option<&str>,
        required: Option<&JavaVersion>,
        on_progress: &impl Fn(LaunchProgress),
    ) -> Result<PathBuf> {
        if let Some(path) = configured {
            let path = PathBuf::from(path);
            return match check_java(&path).await {
                Some(_) => Ok(path),
                None => Err(Error::InvalidInput(format!(
                    "{} is not a working Java",
                    path.display()
                ))),
            };
        }
        let required_major = required.map_or(DEFAULT_JAVA_MAJOR, |java| java.major_version);
        let installs = java::detect_java_installs(&self.paths.java()).await;
        if let Some(install) = java::pick_java(&installs, required_major) {
            return Ok(PathBuf::from(&install.path));
        }
        if !self.settings.java.auto_download {
            return Err(Error::NoCompatibleJava(required_major));
        }
        let component = required
            .and_then(|java| java.component.as_deref())
            .unwrap_or(java::LEGACY_COMPONENT);
        java::install_runtime(
            self.http,
            component,
            &self.paths.java(),
            self.settings.network.concurrent_downloads as usize,
            |progress| on_progress(LaunchProgress::Downloading(progress)),
        )
        .await
    }
}

impl LaunchCommand {
    pub fn spawn(&self) -> Result<Child> {
        std::fs::create_dir_all(&self.working_dir)
            .map_err(|err| Error::io(&self.working_dir, err))?;
        Command::new(&self.java)
            .args(&self.args)
            .current_dir(&self.working_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|err| Error::io(&self.java, err))
    }
}
