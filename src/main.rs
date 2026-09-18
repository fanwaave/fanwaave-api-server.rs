#![forbid(unsafe_code)]

use fanwaave_api_server::{config::ApiConfig, error::StartupError, flags, server};
use fanwaave_lib_core::fanwaave_config::{parse_fanwaave_config, resolve_fanwaave_config};

fn main() -> Result<(), StartupError> {
    let applied = flags::resolve_sources().map_err(StartupError::Flags)?;
    let domain_text =
        std::fs::read_to_string(".fanwaave-cfg.toml").map_err(StartupError::DomainRead)?;
    let domain = parse_fanwaave_config(&domain_text)
        .map_err(|error| StartupError::DomainParse(error.to_string()))?;
    let resolved = resolve_fanwaave_config(
        &domain,
        &applied.ambient,
        &applied.argv_overrides,
    )
    .map_err(|error| StartupError::DomainResolve(error.to_string()))?;
    let cfg = ApiConfig::from_sources(&applied.merged, &resolved).map_err(StartupError::ApiConfig)?;
    server::run(&cfg)?;
    Ok(())
}
