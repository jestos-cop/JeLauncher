use std::time::{Instant, SystemTime, UNIX_EPOCH};

use jelauncher_core::instance::Instance;
use jelauncher_core::launch::{LaunchOptions, Launcher};
use jelauncher_core::logs::{LogLevel, detect_level};
use tauri::{AppHandle, Manager, State};
use tauri_specta::Event;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::sync::oneshot;

use crate::error::{CommandError, CommandResult};
use crate::events::{InstanceExitedEvent, InstanceLogEvent, LaunchProgressEvent};
use crate::state::AppState;

#[tauri::command]
#[specta::specta]
pub async fn launch_instance(
    app: AppHandle,
    state: State<'_, AppState>,
    instance_id: String,
    options: LaunchOptions,
) -> CommandResult<()> {
    let settings = state
        .settings
        .read()
        .map_err(|_| CommandError::poisoned())?
        .clone();
    let account = super::accounts::launch_account(&state).await?;
    let mut instance = Instance::open(&state.paths.instances(), &instance_id)?;

    {
        let mut running = state.running.lock().map_err(|_| CommandError::poisoned())?;
        if running.contains_key(&instance_id) {
            return Err(CommandError::new("this instance is already running"));
        }
        running.insert(instance_id.clone(), None);
    }
    let launcher = Launcher {
        paths: &state.paths,
        meta: &state.meta,
        http: &state.http,
        settings: &settings,
    };
    let prepared = launcher
        .prepare(&instance, &account, &options, |progress| {
            let _ = LaunchProgressEvent {
                instance_id: instance_id.clone(),
                progress,
            }
            .emit(&app);
        })
        .await;
    let started = match prepared {
        Ok(command) => match command.run_pre_launch().await {
            Ok(()) => command.spawn().map(|child| (command, child)),
            Err(err) => Err(err),
        },
        Err(err) => Err(err),
    };
    let (command, mut child) = match started {
        Ok(started) => started,
        Err(err) => {
            mark_stopped(&app, &instance_id);
            return Err(err.into());
        }
    };

    let (kill_tx, kill_rx) = oneshot::channel();
    if let Ok(mut running) = state.running.lock() {
        running.insert(instance_id.clone(), Some(kill_tx));
    }
    instance.config.last_launch_time = unix_millis();
    instance.save()?;

    if let Some(stdout) = child.stdout.take() {
        tauri::async_runtime::spawn(pump_output(app.clone(), instance_id.clone(), stdout));
    }
    if let Some(stderr) = child.stderr.take() {
        tauri::async_runtime::spawn(pump_output(app.clone(), instance_id.clone(), stderr));
    }
    let record_time = settings.launcher.record_game_time;
    tauri::async_runtime::spawn(async move {
        let started = Instant::now();
        let status = tokio::select! {
            status = child.wait() => status,
            _ = kill_rx => {
                let _ = child.kill().await;
                child.wait().await
            }
        };
        let exit_code = status.ok().and_then(|status| status.code());
        log::info!("{} exited with {exit_code:?}", instance.id);
        if record_time {
            instance.config.total_time_played +=
                i64::try_from(started.elapsed().as_secs()).unwrap_or(0);
            let _ = instance.save();
        }
        if let Err(err) = command.run_post_exit().await {
            let _ = InstanceLogEvent {
                instance_id: instance.id.clone(),
                line: format!("Post-exit command failed: {err}"),
                level: LogLevel::Launcher,
            }
            .emit(&app);
        }
        mark_stopped(&app, &instance.id);
        let _ = InstanceExitedEvent {
            instance_id: instance.id.clone(),
            exit_code,
        }
        .emit(&app);
    });
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn kill_instance(state: State<'_, AppState>, instance_id: String) -> CommandResult<()> {
    let kill = state
        .running
        .lock()
        .map_err(|_| CommandError::poisoned())?
        .get_mut(&instance_id)
        .and_then(Option::take);
    match kill {
        Some(kill) => {
            let _ = kill.send(());
            Ok(())
        }
        None => Err(CommandError::new("this instance is not running")),
    }
}

#[tauri::command]
#[specta::specta]
pub fn list_running_instances(state: State<'_, AppState>) -> CommandResult<Vec<String>> {
    let running = state.running.lock().map_err(|_| CommandError::poisoned())?;
    Ok(running.keys().cloned().collect())
}

async fn pump_output(app: AppHandle, instance_id: String, stream: impl AsyncRead + Unpin) {
    let mut lines = BufReader::new(stream).lines();
    let mut level = LogLevel::Unknown;
    while let Ok(Some(line)) = lines.next_line().await {
        level = detect_level(&line, level);
        let _ = InstanceLogEvent {
            instance_id: instance_id.clone(),
            line,
            level,
        }
        .emit(&app);
    }
}

fn mark_stopped(app: &AppHandle, instance_id: &str) {
    if let Ok(mut running) = app.state::<AppState>().running.lock() {
        running.remove(instance_id);
    }
}

fn unix_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| {
            i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX)
        })
}
