#![allow(unused)]

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub primary: PrimaryConfig,
    pub server: ServerConfig,
    pub mqtt: MQTTConfig,
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

#[derive(Debug, Clone, Deserialize)]
pub struct MQTTConfig {
    #[serde(default = "default_mqtt_broker_uri")]
    pub broker_uri: String,
    pub username: Option<String>,
    pub password: Option<String>,
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
