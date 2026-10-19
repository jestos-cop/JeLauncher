use serde::Serialize;

#[derive(Debug, Serialize, specta::Type)]
pub struct CommandError {
    message: String,
    kind: ErrorKind,
}

/// Errors the UI handles differently from a plain failure message.
#[derive(Debug, PartialEq, Eq, Serialize, specta::Type)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ErrorKind {
    Failed,
    /// The user stopped the task, so it is not reported as a failure.
    Cancelled,
    /// The modpack needs files from these hosts; retry with them allowed to continue.
    UntrustedHosts {
        hosts: Vec<String>,
    },
}

impl From<jelauncher_core::Error> for CommandError {
    fn from(err: jelauncher_core::Error) -> Self {
        match err {
            jelauncher_core::Error::UntrustedHosts(hosts) => Self {
                message: jelauncher_core::Error::UntrustedHosts(hosts.clone()).to_string(),
                kind: ErrorKind::UntrustedHosts { hosts },
            },
            err => Self::new(err.to_string()),
        }
    }
}

impl CommandError {
    /// Every error shown to the user also goes to the launcher log.
    pub fn new(message: impl Into<String>) -> Self {
        let message = message.into();
        log::warn!("{message}");
        Self {
            message,
            kind: ErrorKind::Failed,
        }
    }

    pub fn cancelled() -> Self {
        Self {
            message: "cancelled".into(),
            kind: ErrorKind::Cancelled,
        }
    }

    #[cfg(test)]
    pub fn is_cancelled(&self) -> bool {
        self.kind == ErrorKind::Cancelled
    }

    pub fn poisoned() -> Self {
        Self::new("internal state lock was poisoned")
    }
}

pub type CommandResult<T> = Result<T, CommandError>;
