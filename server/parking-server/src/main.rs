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
use parking_server::app;

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

    let server = app::ServerBuilder::new(config).build().await?;
    server.run(graceful_shutdown()).await?;
    Ok(())
}
async fn graceful_shutdown() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

        let mut terminate = match signal(SignalKind::terminate()) {
            Ok(signal) => signal,
            Err(error) => {
                tracing::error!(error = %error, "failed to listen for SIGTERM");

                if let Err(error) = tokio::signal::ctrl_c().await {
                    tracing::error!(
                        error = %error,
                        "failed to listen for shutdown signal"
                    );
                }

                return;
            }
        };

        let mut interrupt = match signal(SignalKind::interrupt()) {
            Ok(signal) => signal,
            Err(error) => {
                tracing::error!(error = %error, "failed to listen for SIGINT");

                if let Err(error) = tokio::signal::ctrl_c().await {
                    tracing::error!(
                        error = %error,
                        "failed to listen for shutdown signal"
                    );
                }

                return;
            }
        };

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {},
            _ = terminate.recv() => {},
            _ = interrupt.recv() => {},
        }

        tracing::info!("shutdown signal received");
    }

    #[cfg(not(unix))]
    {
        if let Err(error) = tokio::signal::ctrl_c().await {
            tracing::error!(
                error = %error,
                "failed to listen for shutdown signal"
            );

            return;
        }

        tracing::info!("shutdown signal received");
    }
}
