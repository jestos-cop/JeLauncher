use std::future::Future;

use tokio::sync::oneshot;

use crate::error::{CommandError, CommandResult};

/// Runs `task` until it finishes or `cancel` fires. Dropping the task stops its downloads
/// and child processes; a dropped sender never cancels.
pub async fn until_cancelled<T>(
    task: impl Future<Output = jelauncher_core::Result<T>>,
    cancel: &mut oneshot::Receiver<()>,
) -> CommandResult<T> {
    tokio::select! {
        result = task => Ok(result?),
        Ok(()) = cancel => Err(CommandError::cancelled()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn cancels_a_pending_task() {
        let (cancel_tx, mut cancel_rx) = oneshot::channel();
        let _ = cancel_tx.send(());
        let result = until_cancelled(
            std::future::pending::<jelauncher_core::Result<()>>(),
            &mut cancel_rx,
        )
        .await;
        assert!(result.is_err_and(|err| err.is_cancelled()));
    }

    #[tokio::test]
    async fn a_dropped_sender_lets_the_task_finish() {
        let (cancel_tx, mut cancel_rx) = oneshot::channel::<()>();
        drop(cancel_tx);
        let result = until_cancelled(async { Ok(7) }, &mut cancel_rx).await;
        assert!(result.is_ok_and(|value| value == 7));
    }
}
