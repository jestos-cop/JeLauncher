mod args;
mod options;

use std::path::{Path, PathBuf};
use std::process::Stdio;

use serde::Serialize;
use tokio::process::{Child, Command};

pub use args::{ArgsInput, LAUNCHER_NAME, build_arguments};
pub use options::{LaunchOptions, QuickPlay};

use crate::auth::Account;
use crate::forge::{ForgeInstaller, InstallProgress};
use crate::instance::Instance;
use crate::java::{self, check_java};
use crate::meta::MetaService;
use crate::minecraft::{
    Environment, GameFiles, JavaVersion, LoaderProfile, LoaderSpec, VersionJson, artifact_download,
    client_jar_path,
};
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
    InstallingLoader,
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
    pub env: Vec<(String, String)>,
    pub wrapper: Vec<String>,
    pub pre_launch: Option<String>,
    pub post_exit: Option<String>,
}

impl Launcher<'_> {
    /// Makes sure every game file is present and builds the command line.
    pub async fn prepare(
        &self,
        instance: &Instance,
        account: &Account,
        options: &LaunchOptions,
        on_progress: impl Fn(LaunchProgress),
    ) -> Result<LaunchCommand> {
        on_progress(LaunchProgress::Preparing);
        let game_version = &instance.config.minecraft_version;
        let mut version = self.meta.version_json(game_version).await?;
        let settings = instance.config.settings.resolve(self.settings);
        ensure_enough_memory(settings.max_memory_mb).await?;
        let java = self
            .find_java(
                settings.java_path.as_deref(),
                version.java_version.as_ref(),
                &on_progress,
            )
            .await?;
        if let Some(loader) = &instance.config.loader {
            let profile = if loader.kind.uses_installer() {
                self.install_forge(&version, loader, &java, &on_progress)
                    .await?
            } else {
                self.meta.loader_profile(loader, game_version).await?
            };
            version.apply_loader(profile);
        }
        let env = Environment::current(options.features(&version, !settings.window_maximized));

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
            options,
        });
        let commands = &settings.commands;
        let wrapper = shlex::split(&commands.wrapper).ok_or_else(|| {
            Error::InvalidInput(format!(
                "cannot parse wrapper command: {}",
                commands.wrapper
            ))
        })?;
        let mut env = vec![
            ("INST_NAME".to_owned(), instance.config.name.clone()),
            ("INST_ID".to_owned(), instance.id.clone()),
            (
                "INST_DIR".to_owned(),
                instance.root.to_string_lossy().into_owned(),
            ),
            (
                "INST_MC_DIR".to_owned(),
                game_dir.to_string_lossy().into_owned(),
            ),
            ("INST_JAVA".to_owned(), java.to_string_lossy().into_owned()),
        ];
        env.extend(
            commands
                .environment
                .iter()
                .map(|(key, value)| (key.clone(), value.clone())),
        );
        let non_empty =
            |command: &String| Some(command.trim().to_owned()).filter(|c| !c.is_empty());

        on_progress(LaunchProgress::Starting);
        Ok(LaunchCommand {
            java,
            args,
            working_dir: game_dir,
            env,
            wrapper,
            pre_launch: non_empty(&commands.pre_launch),
            post_exit: non_empty(&commands.post_exit),
        })
    }

    async fn install_forge(
        &self,
        vanilla: &VersionJson,
        loader: &LoaderSpec,
        java: &Path,
        on_progress: &impl Fn(LaunchProgress),
    ) -> Result<LoaderProfile> {
        // The installer patches the vanilla client jar, so it has to be present first.
        let client_jar = client_jar_path(self.paths, &vanilla.id);
        self.http
            .download(&artifact_download(
                &vanilla.downloads.client,
                client_jar.clone(),
            ))
            .await?;
        let installer = ForgeInstaller {
            http: self.http,
            paths: self.paths,
            java,
            concurrency: self.settings.network.concurrent_downloads as usize,
        };
        installer
            .install(loader, &vanilla.id, &client_jar, |progress| {
                on_progress(match progress {
                    InstallProgress::Downloading(progress) => LaunchProgress::Downloading(progress),
                    InstallProgress::Processing => LaunchProgress::InstallingLoader,
                });
            })
            .await
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
        let (program, prefix) = match self.wrapper.split_first() {
            Some((program, rest)) => (PathBuf::from(program), rest.to_vec()),
            None => (self.java.clone(), Vec::new()),
        };
        let mut command = Command::new(&program);
        command.args(&prefix);
        if !self.wrapper.is_empty() {
            command.arg(&self.java);
        }
        command
            .args(&self.args)
            .envs(self.env.iter().cloned())
            .current_dir(&self.working_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|err| Error::io(&program, err))
    }

    /// Runs the pre-launch command; a non-zero exit cancels the launch.
    pub async fn run_pre_launch(&self) -> Result<()> {
        match &self.pre_launch {
            Some(command) => self.run_hook(command).await,
            None => Ok(()),
        }
    }

    pub async fn run_post_exit(&self) -> Result<()> {
        match &self.post_exit {
            Some(command) => self.run_hook(command).await,
            None => Ok(()),
        }
    }

    async fn run_hook(&self, command: &str) -> Result<()> {
        let (shell, flag) = if cfg!(windows) {
            ("cmd", "/C")
        } else {
            ("sh", "-c")
        };
        let status = Command::new(shell)
            .args([flag, command])
            .envs(self.env.iter().cloned())
            .current_dir(&self.working_dir)
            .stdin(Stdio::null())
            .status()
            .await
            .map_err(|err| Error::io(shell, err))?;
        if status.success() {
            Ok(())
        } else {
            Err(Error::InvalidInput(format!(
                "`{command}` exited with {status}"
            )))
        }
    }
}

/// Refuses to start a game that may use more memory than the computer has.
async fn ensure_enough_memory(max_memory_mb: u32) -> Result<()> {
    match crate::system::total_memory_mb().await {
        Some(total) if u64::from(max_memory_mb) > total => Err(Error::InvalidInput(format!(
            "maximum memory is {max_memory_mb} MiB but this computer has only {total} MiB; lower it in the settings"
        ))),
        _ => Ok(()),
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    fn command(pre_launch: &str, wrapper: Vec<String>) -> LaunchCommand {
        LaunchCommand {
            java: PathBuf::from("echo"),
            args: vec!["game-args".into()],
            working_dir: std::env::temp_dir(),
            env: vec![("INST_NAME".into(), "My Pack".into())],
            wrapper,
            pre_launch: Some(pre_launch.into()),
            post_exit: None,
        }
    }

    #[tokio::test]
    async fn hooks_see_instance_variables_and_fail_on_non_zero_exit() {
        assert!(
            command(r#"test "$INST_NAME" = "My Pack""#, vec![])
                .run_pre_launch()
                .await
                .is_ok()
        );
        assert!(command("exit 3", vec![]).run_pre_launch().await.is_err());
    }

    #[tokio::test]
    async fn wrapper_prefixes_the_java_invocation() -> Result<()> {
        let output = command("true", vec!["echo".into(), "wrapped".into()])
            .spawn()?
            .wait_with_output()
            .await
            .map_err(|err| Error::io("echo", err))?;
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            "wrapped echo game-args"
        );
        Ok(())
    }
}
