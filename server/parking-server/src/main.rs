mod config;
mod http;
mod mqtt;
mod protocol;
mod state;

use std::net::SocketAddr;

use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::{config::Config, state::new_shared_store};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config = Config::from_env()?;

    info!(
        broker = %config.broker_uri,
        http = %format!("{}:{}", config.http_host, config.http_port),
        "starting parking server"
    );

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

    let app = http::router(store);

    let address = format!("{}:{}", config.http_host, config.http_port).parse::<SocketAddr>()?;

    let listener = tokio::net::TcpListener::bind(address).await?;

    info!("HTTP server listening on {address}");

    axum::serve(listener, app).await?;

    Ok(())
}
