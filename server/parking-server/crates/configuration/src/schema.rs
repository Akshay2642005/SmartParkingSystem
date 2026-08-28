use serde::Deserialize;

use crate::secret::Secret;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub primary: PrimaryConfig,
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub mqtt: MqttConfig,
    #[serde(default)]
    pub telemetry: TelemetryConfig,
    #[serde(default)]
    pub store: Option<DatabaseConfig>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrimaryConfig {
    #[serde(default = "default_env")]
    pub env: String,
    #[serde(default = "default_name")]
    pub name: String,
}

impl Default for PrimaryConfig {
    fn default() -> Self {
        Self {
            env: default_env(),
            name: default_name(),
        }
    }
}

impl PrimaryConfig {
    #[must_use]
    pub fn is_development(&self) -> bool {
        self.env == "development"
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_read_timeout_secs")]
    pub read_timeout_secs: u64,
    #[serde(default = "default_write_timeout_secs")]
    pub write_timeout_secs: u64,
    #[serde(default = "default_idle_timeout_secs")]
    pub idle_timeout_secs: u64,
    #[serde(default = "default_request_timeout_secs")]
    pub request_timeout_secs: u64,
    #[serde(default = "default_shutdown_timeout_secs")]
    pub shutdown_timeout_secs: u64,
    #[serde(default = "default_min_graceful_shutdown_secs")]
    pub min_graceful_shutdown_secs: u64,
    #[serde(default = "default_max_body_size_bytes")]
    pub max_body_size_bytes: usize,
    #[serde(default = "default_cors_allowed_origins")]
    pub cors_allowed_origins: Vec<String>,
    #[serde(default = "default_path_prefix")]
    pub path_prefix: String,
    #[serde(default)]
    pub rate_limit: RateLimitConfig,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            read_timeout_secs: default_read_timeout_secs(),
            write_timeout_secs: default_write_timeout_secs(),
            idle_timeout_secs: default_idle_timeout_secs(),
            request_timeout_secs: default_request_timeout_secs(),
            shutdown_timeout_secs: default_shutdown_timeout_secs(),
            min_graceful_shutdown_secs: default_min_graceful_shutdown_secs(),
            max_body_size_bytes: default_max_body_size_bytes(),
            cors_allowed_origins: default_cors_allowed_origins(),
            path_prefix: default_path_prefix(),
            rate_limit: RateLimitConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RateLimitConfig {
    #[serde(default = "default_rate_limit_enabled")]
    pub enabled: bool,
    #[serde(default = "default_rate_limit_requests_per_minute")]
    pub requests_per_minute: u64,
    #[serde(default = "default_rate_limit_burst_size")]
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: default_rate_limit_enabled(),
            requests_per_minute: default_rate_limit_requests_per_minute(),
            burst_size: default_rate_limit_burst_size(),
        }
    }
}

impl RateLimitConfig {
    #[must_use]
    pub fn replenish_interval_ms(&self) -> u64 {
        (60_000 / self.requests_per_minute.max(1)).max(1)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MqttConfig {
    #[serde(default = "default_mqtt_broker_uri")]
    pub broker_uri: String,
    #[serde(default = "default_mqtt_client_id")]
    pub client_id: String,
    #[serde(default = "default_mqtt_keep_alive_secs")]
    pub keep_alive_secs: u64,
    #[serde(default = "default_mqtt_reconnect_delay_secs")]
    pub reconnect_delay_secs: u64,
    #[serde(default = "default_mqtt_capacity")]
    pub capacity: usize,
    pub username: Option<Secret>,
    pub password: Option<Secret>,
}

impl Default for MqttConfig {
    fn default() -> Self {
        Self {
            broker_uri: default_mqtt_broker_uri(),
            client_id: default_mqtt_client_id(),
            keep_alive_secs: default_mqtt_keep_alive_secs(),
            reconnect_delay_secs: default_mqtt_reconnect_delay_secs(),
            capacity: default_mqtt_capacity(),
            username: None,
            password: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatabaseConfig {
    pub url: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,
    #[serde(default = "default_connect_timeout_secs")]
    pub connect_timeout_secs: u64,
    #[serde(default = "default_acquire_timeout_secs")]
    pub acquire_timeout_secs: u64,
    #[serde(default = "default_store_idle_timeout_secs")]
    pub idle_timeout_secs: u64,
    #[serde(default = "default_max_lifetime_secs")]
    pub max_lifetime_secs: u64,
    #[serde(default = "default_sqlx_logging")]
    pub sqlx_logging: bool,
    #[serde(default = "default_migrate_on_start")]
    pub migrate_on_start: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogFormat {
    #[default]
    Compact,
    Pretty,
    Json,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TelemetryConfig {
    #[serde(default = "default_service_name")]
    pub service_name: String,
    #[serde(default = "default_environment")]
    pub environment: String,
    #[serde(default = "default_filter")]
    pub filter: String,
    #[serde(default)]
    pub format: LogFormat,
    #[serde(default = "default_ansi")]
    pub ansi: bool,
    #[serde(default)]
    pub include_file: bool,
    #[serde(default)]
    pub include_line_number: bool,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            service_name: default_service_name(),
            environment: default_environment(),
            filter: default_filter(),
            format: LogFormat::default(),
            ansi: default_ansi(),
            include_file: false,
            include_line_number: false,
        }
    }
}

fn default_env() -> String {
    "development".to_owned()
}

fn default_name() -> String {
    "parking-server".to_owned()
}

fn default_service_name() -> String {
    "parking-server".to_owned()
}

fn default_environment() -> String {
    "development".to_owned()
}

fn default_filter() -> String {
    "info,tower_http=info".to_owned()
}

fn default_ansi() -> bool {
    std::io::IsTerminal::is_terminal(&std::io::stderr())
}

fn default_host() -> String {
    "0.0.0.0".to_owned()
}

fn default_port() -> u16 {
    8080
}

fn default_read_timeout_secs() -> u64 {
    15
}
fn default_write_timeout_secs() -> u64 {
    15
}
fn default_idle_timeout_secs() -> u64 {
    60
}
fn default_request_timeout_secs() -> u64 {
    15
}

fn default_shutdown_timeout_secs() -> u64 {
    30
}

fn default_min_graceful_shutdown_secs() -> u64 {
    0
}

fn default_max_body_size_bytes() -> usize {
    10 * 1024 * 1024 // 10 MB
}

fn default_cors_allowed_origins() -> Vec<String> {
    vec!["http://localhost:3000".to_owned()]
}

fn default_path_prefix() -> String {
    "/api/v1".to_owned()
}

fn default_rate_limit_enabled() -> bool {
    true
}

fn default_rate_limit_requests_per_minute() -> u64 {
    100
}

fn default_rate_limit_burst_size() -> u32 {
    20
}

fn default_mqtt_broker_uri() -> String {
    "mqtt://localhost:1883".to_owned()
}

fn default_mqtt_client_id() -> String {
    "parking-server".to_owned()
}

fn default_mqtt_keep_alive_secs() -> u64 {
    30
}

fn default_mqtt_reconnect_delay_secs() -> u64 {
    1
}

fn default_mqtt_capacity() -> usize {
    32
}

fn default_max_connections() -> u32 {
    4
}

fn default_min_connections() -> u32 {
    1
}

fn default_connect_timeout_secs() -> u64 {
    30
}

fn default_acquire_timeout_secs() -> u64 {
    30
}

fn default_store_idle_timeout_secs() -> u64 {
    30
}

fn default_max_lifetime_secs() -> u64 {
    1800
}

fn default_sqlx_logging() -> bool {
    true
}

fn default_migrate_on_start() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_format_deserializes_the_documented_tokens() {
        for (token, expected) in [
            ("\"compact\"", LogFormat::Compact),
            ("\"pretty\"", LogFormat::Pretty),
            ("\"json\"", LogFormat::Json),
        ] {
            assert_eq!(serde_json::from_str::<LogFormat>(token).unwrap(), expected);
        }

        assert_eq!(LogFormat::default(), LogFormat::Compact);
    }

    #[test]
    fn a_partial_file_keeps_defaults_for_everything_else() {
        let config: Config = serde_yaml::from_str(
            "primary:\n  name: parking-server\nmqtt:\n  broker_uri: mqtt://broker:1883\n",
        )
        .unwrap();

        assert_eq!(config.mqtt.broker_uri, "mqtt://broker:1883");
        assert_eq!(config.mqtt.client_id, "parking-server");
        assert_eq!(config.mqtt.keep_alive_secs, 30);
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.path_prefix, "/api/v1");
        assert_eq!(config.telemetry.format, LogFormat::Compact);
        assert!(config.store.is_none());
    }

    #[test]
    fn rate_limit_converts_requests_per_minute_to_a_replenish_interval() {
        let mut config = RateLimitConfig::default();
        assert_eq!(config.requests_per_minute, 100);
        assert_eq!(config.replenish_interval_ms(), 600);

        config.requests_per_minute = 60;
        assert_eq!(config.replenish_interval_ms(), 1_000);

        // Never zero, whatever the configured rate.
        config.requests_per_minute = 1_000_000;
        assert_eq!(config.replenish_interval_ms(), 1);
    }

    #[test]
    fn an_unknown_key_in_a_section_is_rejected() {
        assert!(serde_yaml::from_str::<Config>("server:\n  prot: 8080\n").is_err());
    }
}
