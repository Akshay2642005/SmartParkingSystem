//! Startup validation: every rule here turns a misconfiguration into a boot
//! failure naming the offending field, instead of a confusing runtime error.

use crate::schema::{Config, DatabaseConfig, MqttConfig, ServerConfig};

pub(crate) fn validate_configuration(config: &Config) -> anyhow::Result<()> {
    ensure_not_blank(&config.primary.env, "primary.env")?;
    ensure_not_blank(&config.primary.name, "primary.name")?;

    validate_server(&config.server)?;
    validate_mqtt(&config.mqtt)?;
    validate_telemetry(config)?;

    if let Some(store) = &config.store {
        validate_store(store)?;
    }

    Ok(())
}

fn validate_server(server: &ServerConfig) -> anyhow::Result<()> {
    if !server.path_prefix.starts_with('/') {
        anyhow::bail!(
            "server.path_prefix must start with '/' (got {:?})",
            server.path_prefix
        );
    }

    if server.request_timeout_secs == 0 {
        anyhow::bail!("server.request_timeout_secs must be greater than 0");
    }

    if server.max_body_size_bytes == 0 {
        anyhow::bail!("server.max_body_size_bytes must be greater than 0");
    }

    for origin in &server.cors_allowed_origins {
        if origin != "*" && url::Url::parse(origin).is_err() {
            anyhow::bail!("server.cors_allowed_origins contains an invalid origin: {origin}");
        }
    }

    if server.rate_limit.enabled {
        if server.rate_limit.requests_per_minute == 0 {
            anyhow::bail!(
                "server.rate_limit.requests_per_minute must be greater than 0 when enabled"
            );
        }

        if server.rate_limit.burst_size == 0 {
            anyhow::bail!("server.rate_limit.burst_size must be greater than 0 when enabled");
        }
    }

    Ok(())
}

/// The broker URI is the one field that silently breaks device ingest when
/// wrong, so it is parsed (not just non-blank checked) at startup.
fn validate_mqtt(mqtt: &MqttConfig) -> anyhow::Result<()> {
    ensure_not_blank(&mqtt.client_id, "mqtt.client_id")?;

    let uri = url::Url::parse(&mqtt.broker_uri)
        .map_err(|error| anyhow::anyhow!("mqtt.broker_uri is not a valid URI: {error}"))?;

    if !matches!(uri.scheme(), "mqtt" | "mqtts") {
        anyhow::bail!(
            "mqtt.broker_uri scheme must be mqtt or mqtts (got {:?})",
            uri.scheme()
        );
    }

    if uri.host_str().is_none_or(str::is_empty) {
        anyhow::bail!("mqtt.broker_uri must carry a host");
    }

    if mqtt.keep_alive_secs == 0 {
        anyhow::bail!("mqtt.keep_alive_secs must be greater than 0");
    }

    if mqtt.capacity == 0 {
        anyhow::bail!("mqtt.capacity must be greater than 0");
    }

    if mqtt.password.is_some() && mqtt.username.is_none() {
        anyhow::bail!("mqtt.password is set without mqtt.username");
    }

    Ok(())
}

fn validate_telemetry(config: &Config) -> anyhow::Result<()> {
    ensure_not_blank(&config.telemetry.filter, "telemetry.filter")?;
    ensure_not_blank(&config.telemetry.service_name, "telemetry.service_name")?;

    Ok(())
}

fn validate_store(store: &DatabaseConfig) -> anyhow::Result<()> {
    ensure_not_blank(&store.url, "store.url")?;

    let url = url::Url::parse(&store.url)
        .map_err(|error| anyhow::anyhow!("store.url is not a valid URI: {error}"))?;

    if !matches!(url.scheme(), "postgres" | "postgresql") {
        anyhow::bail!(
            "store.url must be a postgres:// URI (got scheme {:?})",
            url.scheme()
        );
    }

    if store.max_connections == 0 {
        anyhow::bail!("store.max_connections must be greater than 0");
    }

    if store.min_connections > store.max_connections {
        anyhow::bail!("store.min_connections cannot exceed store.max_connections");
    }

    Ok(())
}

fn ensure_not_blank(value: &str, field: &str) -> anyhow::Result<()> {
    if value.trim().is_empty() {
        anyhow::bail!("{field} cannot be blank");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> Config {
        Config::default()
    }

    #[test]
    fn defaults_are_valid() {
        validate_configuration(&config()).unwrap();
    }

    #[test]
    fn broker_uri_scheme_is_enforced() {
        let mut config = config();
        config.mqtt.broker_uri = "http://localhost:1883".into();

        let error = validate_configuration(&config).unwrap_err().to_string();

        assert!(error.contains("mqtt.broker_uri scheme"), "{error}");
    }

    #[test]
    fn broker_uri_must_parse() {
        let mut config = config();
        config.mqtt.broker_uri = "not a uri".into();

        assert!(validate_configuration(&config).is_err());
    }

    #[test]
    fn store_url_must_be_postgres() {
        let mut config = config();
        config.store = Some(DatabaseConfig {
            url: "mysql://localhost/parking".into(),
            max_connections: 4,
            min_connections: 1,
            connect_timeout_secs: 30,
            acquire_timeout_secs: 30,
            idle_timeout_secs: 30,
            max_lifetime_secs: 1800,
            sqlx_logging: false,
            migrate_on_start: true,
        });

        let error = validate_configuration(&config).unwrap_err().to_string();

        assert!(error.contains("store.url must be a postgres"), "{error}");
    }

    #[test]
    fn path_prefix_must_be_absolute() {
        let mut config = config();
        config.server.path_prefix = "api/v1".into();

        assert!(validate_configuration(&config).is_err());
    }

    #[test]
    fn broker_password_without_username_is_rejected() {
        let mut config = config();
        config.mqtt.password = Some("pw".into());

        let error = validate_configuration(&config).unwrap_err().to_string();

        assert!(error.contains("without mqtt.username"), "{error}");
    }
}
