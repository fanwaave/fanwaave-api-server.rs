#![forbid(unsafe_code)]

use fanwaave_api_server::{config::ApiConfig, error::StartupError, flags, server};
use fanwaave_lib_core::fanwaave_config::{parse_fanwaave_config, resolve_fanwaave_config};

#[tokio::main]
async fn main() -> Result<(), StartupError> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "fanwaave_api_server=info,info".into()),
        )
        .json()
        .init();

    let applied = flags::resolve_sources().map_err(|error| StartupError::Flags(error.to_string()))?;
    let domain_text = std::fs::read_to_string(".fanwaave-cfg.toml")
        .map_err(StartupError::DomainConfigFile)?;
    let domain = parse_fanwaave_config(&domain_text)
        .map_err(|error| StartupError::DomainConfig(error.to_string()))?;
    let resolved = resolve_fanwaave_config(
        &domain,
        &applied.ambient,
        &applied.argv_overrides,
    )
    .map_err(|error| StartupError::DomainResolution(error.to_string()))?;
    let cfg = ApiConfig::from_sources(&applied.merged, &resolved)
        .map_err(StartupError::ApiConfig)?;
    server::run(&cfg).await?;
    Ok(())
}
