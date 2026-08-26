use std::sync::Arc;

use crate::error::TraceInitError;
use configuration::{Config, LogFormat};
use tracing::Subscriber;
use tracing_error::ErrorLayer;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::time::ChronoUtc;
use tracing_subscriber::{EnvFilter, Layer, fmt, prelude::*};

#[derive(Debug)]
pub struct Telemetry;

pub fn init_tracing(config: Arc<Config>) -> Result<Telemetry, TraceInitError> {
    let _ = tracing_log::LogTracer::init();

    let filter =
        EnvFilter::try_new(&config.telemetry.filter).map_err(TraceInitError::InvalidFilter)?;
    let fmt_layer = build_fmt_layer(&config, filter);

    install(
        tracing_subscriber::registry()
            .with(ErrorLayer::default())
            .with(fmt_layer),
    )?;

    tracing::info!(
        deployment.environment = %config.telemetry.environment,
        service.name = %config.telemetry.service_name,
        "telemetry initialized"
    );

    Ok(Telemetry)
}
fn install<S>(subscriber: S) -> Result<(), TraceInitError>
where
    S: Subscriber + Send + Sync + 'static,
{
    tracing::dispatcher::set_global_default(tracing::Dispatch::new(subscriber))
        .map_err(TraceInitError::InstallSubscriber)
}

fn build_fmt_layer<S>(c: &Config, filter: EnvFilter) -> Box<dyn Layer<S> + Send + Sync>
where
    S: tracing::Subscriber + for<'span> tracing_subscriber::registry::LookupSpan<'span>,
{
    match &c.telemetry.format {
        LogFormat::Compact => base_fmt_layer(c)
            .compact()
            .with_timer(ChronoUtc::new("%Y-%m-%d %H:%M:%S%.3f".to_string()))
            .with_filter(filter)
            .boxed(),

        LogFormat::Pretty => base_fmt_layer(c)
            .pretty()
            .with_timer(ChronoUtc::new("%Y-%m-%d %H:%M:%S%.3f".to_string()))
            .with_filter(filter)
            .boxed(),

        LogFormat::Json => base_fmt_layer(c)
            .json()
            .with_timer(ChronoUtc::rfc_3339())
            .flatten_event(true)
            .with_current_span(true)
            .with_span_list(true)
            .with_filter(filter)
            .boxed(),
    }
}

fn base_fmt_layer<S>(c: &Config) -> fmt::Layer<S>
where
    S: tracing::Subscriber,
    S: for<'span> tracing_subscriber::registry::LookupSpan<'span>,
{
    fmt::layer()
        .with_ansi(c.telemetry.ansi)
        .with_target(false)
        .with_file(c.telemetry.include_file)
        .with_line_number(c.telemetry.include_line_number)
        .with_span_events(FmtSpan::CLOSE)
}
