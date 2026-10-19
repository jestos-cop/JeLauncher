use serde::Serialize;

#[derive(Debug, Serialize, specta::Type)]
pub struct CommandError {
    message: String,
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
        Self { message }
    }

    pub fn poisoned() -> Self {
        Self::new("internal state lock was poisoned")
    }
}

pub type CommandResult<T> = Result<T, CommandError>;
