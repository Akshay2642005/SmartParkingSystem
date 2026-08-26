#![allow(unused)]

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub primary: PrimaryConfig,
    pub server: ServerConfig,
    pub mqtt: MQTTConfig,
    pub telemetry: TelemetryConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PrimaryConfig {
    pub env: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
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
    pub cors_allowed_origins: Vec<String>,
    pub path_prefix: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum LogFormat {
    #[default]
    Compact,
    Pretty,
    Json,
}

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
pub struct MQTTConfig {
    #[serde(default = "default_mqtt_broker_uri")]
    pub broker_uri: String,
    pub username: Option<String>,
    pub password: Option<String>,
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

fn default_service_name() -> String {
    "app".to_owned()
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

fn default_mqtt_broker_uri() -> String {
    "mqtt://localhost:1883".to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_format_deserializes_all_variants() {
        assert_eq!(
            serde_json::from_str::<LogFormat>("\"compact\"").unwrap(),
            LogFormat::Compact
        );
        assert_eq!(
            serde_json::from_str::<LogFormat>("\"pretty\"").unwrap(),
            LogFormat::Pretty
        );
        assert_eq!(
            serde_json::from_str::<LogFormat>("\"json\"").unwrap(),
            LogFormat::Json
        );
    }

    #[test]
    fn log_format_defaults_to_compact() {
        assert_eq!(LogFormat::default(), LogFormat::Compact);
    }

    #[test]
    fn telemetry_config_defaults() {
        let cfg = TelemetryConfig::default();
        assert_eq!(cfg.service_name, "app");
        assert_eq!(cfg.environment, "development");
        assert_eq!(cfg.filter, "info,tower_http=info");
        assert_eq!(cfg.format, LogFormat::Compact);
    }

    #[test]
    fn telemetry_config_partial_yaml() {
        let yaml = r#"
            filter: debug,server=trace
        "#;
        let cfg: TelemetryConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(cfg.filter, "debug,server=trace");
        assert_eq!(cfg.service_name, "app");
        assert_eq!(cfg.format, LogFormat::Compact);
    }

    #[test]
    fn config_full_minimal() {
        let yaml = r#"
            primary:
              env: development
              name: test
            server:
              host: 127.0.0.1
              port: 9000
              cors_allowed_origins: []
              path_prefix: /api/v1
            mqtt:
              broker_uri: mqtt://localhost:1883
            telemetry:
              filter: info
        "#;
        let cfg: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(cfg.primary.name, "test");
        assert_eq!(cfg.server.port, 9000);
        assert_eq!(cfg.mqtt.broker_uri, "mqtt://localhost:1883");
        assert_eq!(cfg.telemetry.filter, "info");
    }
}
