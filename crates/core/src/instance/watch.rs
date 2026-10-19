use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

use notify::{RecursiveMode, Watcher};

use crate::{Error, Result};

/// Changes arriving closer together than this are reported once.
const DEBOUNCE: Duration = Duration::from_millis(300);

/// Keeps watching while alive.
pub struct InstanceWatcher {
    _watcher: notify::RecommendedWatcher,
}

/// Calls `on_change` when instance folders are added, removed or renamed outside the launcher.
pub fn watch_instances(
    instances_dir: &Path,
    on_change: impl Fn() + Send + 'static,
) -> Result<InstanceWatcher> {
    std::fs::create_dir_all(instances_dir).map_err(|err| Error::io(instances_dir, err))?;
    let (sender, receiver) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
        if event.is_ok() {
            let _ = sender.send(());
        }
    })
    .map_err(|err| {
        Error::InvalidInput(format!("cannot watch {}: {err}", instances_dir.display()))
    })?;
    watcher
        .watch(instances_dir, RecursiveMode::NonRecursive)
        .map_err(|err| {
            Error::InvalidInput(format!("cannot watch {}: {err}", instances_dir.display()))
        })?;
    std::thread::spawn(move || {
        while receiver.recv().is_ok() {
            while receiver.recv_timeout(DEBOUNCE).is_ok() {}
            on_change();
        }
    });
    Ok(InstanceWatcher { _watcher: watcher })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    #[test]
    fn reports_new_instance_folders_once() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|err| Error::io("tempdir", err))?;
        let changes = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&changes);
        let _watcher = watch_instances(dir.path(), move || {
            counter.fetch_add(1, Ordering::SeqCst);
        })?;
        std::thread::sleep(Duration::from_millis(200));
        for name in ["a", "b", "c"] {
            let path = dir.path().join(name);
            std::fs::create_dir(&path).map_err(|err| Error::io(&path, err))?;
        }
        std::thread::sleep(Duration::from_secs(2));
        assert!(changes.load(Ordering::SeqCst) >= 1);
        assert!(
            changes.load(Ordering::SeqCst) <= 2,
            "changes were not debounced"
        );
        Ok(())
    }
}
