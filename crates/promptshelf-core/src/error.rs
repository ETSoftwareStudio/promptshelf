use std::path::PathBuf;

/// Every error the core can return.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("{path}: invalid frontmatter: {message}")]
    Frontmatter { path: PathBuf, message: String },

    #[error("{path}: invalid config: {message}")]
    Config { path: PathBuf, message: String },

    #[error("no config found; run `shelf init` first")]
    NoConfig,

    #[error("no library named \"{0}\"")]
    UnknownLibrary(String),

    #[error("no prompt with the ID \"{0}\"")]
    NotFound(String),

    #[error("the ID \"{id}\" matches more than one prompt: {}", .matches.join(", "))]
    Ambiguous { id: String, matches: Vec<String> },

    #[error("{0}")]
    Template(String),

    #[error("{0}")]
    Invalid(String),

    #[error("{0}")]
    Refused(String),

    #[error("{0}")]
    Git(String),
}

impl Error {
    /// A short, stable code for JSON output.
    pub fn code(&self) -> &'static str {
        match self {
            Error::Io { .. } => "io",
            Error::Frontmatter { .. } => "frontmatter",
            Error::Config { .. } => "config",
            Error::NoConfig => "no_config",
            Error::UnknownLibrary(_) => "unknown_library",
            Error::NotFound(_) => "not_found",
            Error::Ambiguous { .. } => "ambiguous",
            Error::Template(_) => "template",
            Error::Invalid(_) => "invalid",
            Error::Refused(_) => "refused",
            Error::Git(_) => "git",
        }
    }

    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Error::Io {
            path: path.into(),
            source,
        }
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
