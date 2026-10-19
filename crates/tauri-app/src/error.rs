use serde::Serialize;

#[derive(Debug, Serialize, specta::Type)]
pub struct CommandError {
    message: String,
}

impl From<jelauncher_core::Error> for CommandError {
    fn from(err: jelauncher_core::Error) -> Self {
        Self {
            message: err.to_string(),
        }
    }
}

pub type CommandResult<T> = Result<T, CommandError>;
