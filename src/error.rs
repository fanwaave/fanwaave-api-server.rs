#![forbid(unsafe_code)]

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("unauthenticated")]
    Unauthenticated,
    #[error("forbidden")]
    Forbidden,
}

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("invalid server configuration: {0}")]
    Configuration(&'static str),
    #[error("database operation failed")]
    Database(#[source] sqlx::Error),
    #[error("I/O operation failed")]
    Io(#[source] std::io::Error),
}

#[derive(Debug, Error)]
pub enum StartupError {
    #[error("flag resolution failed: {0}")]
    Flags(String),
    #[error("cannot read .fanwaave-cfg.toml")]
    DomainConfigFile(#[source] std::io::Error),
    #[error("invalid .fanwaave-cfg.toml: {0}")]
    DomainConfig(String),
    #[error("Fanwaave domain config resolution failed: {0}")]
    DomainResolution(String),
    #[error("Fanwaave API config resolution failed: {0}")]
    ApiConfig(String),
    #[error(transparent)]
    Server(#[from] ServerError),
}
