mod http;
mod mqtt;
mod protocol;
mod state;

use anyhow::Context;
use configuration::load_config;
use std::net::SocketAddr;

use tracing::info;

use crate::state::new_shared_store;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config_dir = std::env::var("CONFIG_DIR").unwrap_or_else(|_| ".".into());

    let config = std::sync::Arc::new(
        load_config(std::path::Path::new(&config_dir)).context("failed to load config")?,
    );

    let _ = telemetry::init_tracing(config.clone()).context("failed to initialize telemetry")?;

    info!(config.environment = %config.primary.env, config.name = %config.primary.name, "config loaded");

    let store = new_shared_store();
    let mqtt_store = store.clone();
    let mqtt_config = config.clone();

    tokio::spawn(async move {
        tracing::info!("starting MQTT subscriber");

        if let Err(error) = mqtt::run(mqtt_config, mqtt_store).await {
            tracing::error!(
                error = %error,
                "MQTT subscriber terminated"
            );
        }
        tracing::error!("MQTT subscriber task exited");
    });

    info!(
        broker = %config.mqtt.broker_uri,
        server = %format!("http://{}:{}", config.server.host, config.server.port),
        "starting parking server"
    );

    let app = http::router(store);
    let address = format!("{}:{}", config.server.host, config.server.port).parse::<SocketAddr>()?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    info!("HTTP server listening on: http://{address}");
    axum::serve(listener, app).await?;

    Ok(())
}
