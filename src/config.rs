#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use fanwaave_lib_core::fanwaave_config::{ConfigValue, ResolvedFanwaaveConfig};

#[derive(Clone, Debug)]
pub struct ApiConfig {
    pub bind: String,
    pub tcp_bind: Option<String>,
    pub nats_url: Option<String>,
}

impl ApiConfig {
    pub fn from_env() -> Self {
        Self::from_map(&std::env::vars().collect())
    }

    pub fn from_map(environment: &BTreeMap<String, String>) -> Self {
        Self {
            bind: environment
                .get("FANWAAVE_API_BIND")
                .cloned()
                .unwrap_or_else(|| "127.0.0.1:8080".into()),
            tcp_bind: environment.get("FANWAAVE_API_TCP_BIND").cloned(),
            nats_url: environment.get("FANWAAVE_NATS_URL").cloned(),
        }
    }

    pub fn from_sources(
        environment: &BTreeMap<String, String>,
        fanwaave: &ResolvedFanwaaveConfig,
    ) -> Result<Self, String> {
        let bind = match fanwaave.binding("bind_addr").map(|binding| binding.value()) {
            Some(ConfigValue::String(value)) => value.clone(),
            Some(_) => return Err("Fanwaave bind_addr binding must resolve as a string".into()),
            None => environment
                .get("FANWAAVE_API_BIND")
                .cloned()
                .unwrap_or_else(|| "127.0.0.1:8080".into()),
        };
        let tcp_bind = match fanwaave.binding("tcp_bind").map(|binding| binding.value()) {
            Some(ConfigValue::String(value)) => Some(value.clone()),
            Some(_) => return Err("Fanwaave tcp_bind binding must resolve as a string".into()),
            None => None,
        };
        let nats_url = match fanwaave.binding("nats_url").map(|binding| binding.value()) {
            Some(ConfigValue::Url(value)) => Some(value.clone()),
            Some(_) => return Err("Fanwaave nats_url binding must resolve as a URL".into()),
            None => None,
        };
        Ok(Self {
            bind,
            tcp_bind,
            nats_url,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fanwaave_lib_core::fanwaave_config::{parse_fanwaave_config, resolve_fanwaave_config};

    const DOMAIN_CONFIG: &str = include_str!("../.fanwaave-cfg.toml");

    #[test]
    fn domain_defaults_and_transport_environment_compose() -> Result<(), String> {
        let policy = parse_fanwaave_config(DOMAIN_CONFIG).map_err(|error| error.to_string())?;
        let ambient = BTreeMap::from([
            (
                "FANWAAVE_API_TCP_BIND".to_owned(),
                "127.0.0.1:8082".to_owned(),
            ),
            (
                "FANWAAVE_NATS_URL".to_owned(),
                "nats://127.0.0.1:4222".to_owned(),
            ),
        ]);
        let resolved = resolve_fanwaave_config(&policy, &ambient, &BTreeMap::new())
            .map_err(|error| error.to_string())?;
        let config = ApiConfig::from_sources(&ambient, &resolved)?;
        assert_eq!(config.bind, "127.0.0.1:8080");
        assert_eq!(config.tcp_bind.as_deref(), Some("127.0.0.1:8082"));
        assert_eq!(config.nats_url.as_deref(), Some("nats://127.0.0.1:4222"));
        Ok(())
    }

    #[test]
    fn secret_nats_binding_rejects_argv_delivery() -> Result<(), String> {
        let policy = parse_fanwaave_config(DOMAIN_CONFIG).map_err(|error| error.to_string())?;
        let resolution = resolve_fanwaave_config(
            &policy,
            &BTreeMap::new(),
            &BTreeMap::from([(
                "FANWAAVE_NATS_URL".to_owned(),
                "nats://user:secret@127.0.0.1:4222".to_owned(),
            )]),
        );
        let error = match resolution {
            Ok(_) => return Err("secret argv unexpectedly crossed the command-line boundary".into()),
            Err(error) => error,
        };
        assert!(error.to_string().contains("may not be supplied through argv"));
        Ok(())
    }
}
