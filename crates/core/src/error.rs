use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not determine the data directory for this platform")]
    NoDataDir,

    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("request to {url} failed: {source}")]
    Http {
        url: String,
        #[source]
        source: reqwest::Error,
    },

    #[error("checksum mismatch for {path}: expected {expected}, got {actual}")]
    Checksum {
        path: PathBuf,
        expected: String,
        actual: String,
    },

    #[error("instance {0} does not exist")]
    UnknownInstance(String),

    #[error("account {0} does not exist")]
    UnknownAccount(String),

    #[error("no installed Java {0} or newer was found")]
    NoCompatibleJava(u32),

    #[error("{0}")]
    InvalidInput(String),

    #[error("unknown Minecraft version {0}")]
    UnknownVersion(String),

    #[error("broken archive {path}: {source}")]
    Archive {
        path: PathBuf,
        #[source]
        source: zip::result::ZipError,
    },

    #[error("invalid JSON in {path}: {source}")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }

    pub(crate) fn http(url: impl Into<String>, source: reqwest::Error) -> Self {
        Self::Http {
            url: url.into(),
            source,
        }
    }

    pub(crate) fn archive(path: impl Into<PathBuf>, source: zip::result::ZipError) -> Self {
        Self::Archive {
            path: path.into(),
            source,
        }
    }

    pub(crate) fn json(path: impl Into<PathBuf>, source: serde_json::Error) -> Self {
        Self::Json {
            path: path.into(),
            source,
        }
    }
}
