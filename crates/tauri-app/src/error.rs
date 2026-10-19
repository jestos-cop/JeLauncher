use serde::Serialize;

#[derive(Debug, Serialize, specta::Type)]
pub struct CommandError {
    message: String,
    /// Set when the user stopped the task, so the UI does not report it as a failure.
    cancelled: bool,
}

impl From<jelauncher_core::Error> for CommandError {
    fn from(err: jelauncher_core::Error) -> Self {
        Self::new(err.to_string())
    }
}

impl CommandError {
    /// Every error shown to the user also goes to the launcher log.
    pub fn new(message: impl Into<String>) -> Self {
        let message = message.into();
        log::warn!("{message}");
        Self {
            message,
            cancelled: false,
        }
    }

    pub fn cancelled() -> Self {
        Self {
            message: "cancelled".into(),
            cancelled: true,
        }
    }

    #[cfg(test)]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }

    pub fn poisoned() -> Self {
        Self::new("internal state lock was poisoned")
    }
}

pub type CommandResult<T> = Result<T, CommandError>;
