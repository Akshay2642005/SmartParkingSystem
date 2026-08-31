#![allow(unused)]

mod domain;
mod events;
mod mqtt;
mod protocol;
mod response;
mod state;
mod store;

use anyhow::Context;
use configuration::load_config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config_dir = std::env::var("CONFIG_DIR").unwrap_or_else(|_| ".".into());

    let config = std::sync::Arc::new(
        load_config(std::path::Path::new(&config_dir)).context("failed to load config")?,
    );

    let _ = telemetry::init_tracing(config.clone()).context("failed to initialize telemetry")?;

    tracing::info!(config.environment = %config.primary.env, config.name = %config.primary.name, "config loaded");

    // let address = format!("{}:{}", config.server.host, config.server.port).parse::<SocketAddr>()?;
    // let listener = tokio::net::TcpListener::bind(address).await?;
    // tracing::info!("HTTP server listening on: http://{address}");

    Ok(())
}
