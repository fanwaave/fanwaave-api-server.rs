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
pub enum StartupError {
    #[error("flags-2-env resolution failed: {0}")]
    Flags(String),
    #[error("cannot read .fanwaave-cfg.toml: {0}")]
    DomainRead(#[source] std::io::Error),
    #[error("invalid .fanwaave-cfg.toml: {0}")]
    DomainParse(String),
    #[error("Fanwaave domain config resolution failed: {0}")]
    DomainResolve(String),
    #[error("Fanwaave API config resolution failed: {0}")]
    ApiConfig(String),
    #[error("Fanwaave server serialization failed: {0}")]
    ServerJson(#[from] serde_json::Error),
}

#[derive(Debug, Error)]
pub enum TransportRuntimeError {
    #[error("JetStream runtime failed: {0}")]
    JetStream(String),
    #[error("transport environment config failed: {0}")]
    Config(String),
    #[error("NATS connection failed: {0}")]
    Connect(String),
}
