use thiserror::Error;

pub type Result<T> = std::result::Result<T, CliError>;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum CliError {
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("HTTP / Request error: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("TOML deserialize error: {0}")]
    TomlDeError(#[from] toml::de::Error),

    #[error("TOML serialize error: {0}")]
    TomlSerError(#[from] toml::ser::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Workspace error: {0}")]
    WorkspaceError(String),

    #[error("Not logged in. Authenticate first")]
    NotLoggedIn,

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}
