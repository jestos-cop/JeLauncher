use std::time::{Instant, SystemTime, UNIX_EPOCH};

use jelauncher_core::instance::Instance;
use jelauncher_core::launch::Launcher;
use jelauncher_core::logs::{LogLevel, detect_level};
use tauri::{AppHandle, Manager, State};
use tauri_specta::Event;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};

use crate::error::{CommandError, CommandResult};
use crate::events::{InstanceExitedEvent, InstanceLogEvent, LaunchProgressEvent};
use crate::state::AppState;

#[tauri::command]
#[specta::specta]
pub async fn launch_instance(
    app: AppHandle,
    state: State<'_, AppState>,
    instance_id: String,
) -> CommandResult<()> {
    let settings = state
        .settings
        .read()
        .map_err(|_| CommandError::poisoned())?
        .clone();
    let account = state
        .accounts
        .read()
        .map_err(|_| CommandError::poisoned())?
        .default_account()
        .cloned()
        .ok_or_else(|| CommandError::new("add an account before launching"))?;
    let mut instance = Instance::open(&state.paths.instances(), &instance_id)?;

    if !state
        .running
        .lock()
        .map_err(|_| CommandError::poisoned())?
        .insert(instance_id.clone())
    {
        return Err(CommandError::new("this instance is already running"));
    }
    let launcher = Launcher {
        paths: &state.paths,
        meta: &state.meta,
        http: &state.http,
        settings: &settings,
    };
    let prepared = launcher
        .prepare(&instance, &account, |progress| {
            let _ = LaunchProgressEvent {
                instance_id: instance_id.clone(),
                progress,
            }
            .emit(&app);
        })
        .await;
    let child = prepared.and_then(|command| command.spawn());
    let mut child = match child {
        Ok(child) => child,
        Err(err) => {
            mark_stopped(&app, &instance_id);
            return Err(err.into());
        }
    };

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
        let exit_code = child.wait().await.ok().and_then(|status| status.code());
        if record_time {
            instance.config.total_time_played +=
                i64::try_from(started.elapsed().as_secs()).unwrap_or(0);
            let _ = instance.save();
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
