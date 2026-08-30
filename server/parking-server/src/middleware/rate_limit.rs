use axum::{body::Body, http::Response, response::IntoResponse};
use configuration::RateLimitConfig;
use governor::middleware::NoOpMiddleware;
use std::sync::Arc;
use tower_governor::{
    GovernorError, governor::GovernorConfigBuilder, key_extractor::PeerIpKeyExtractor,
};

use crate::response::error::AppError;

pub use tower_governor::GovernorLayer;

/// Governor layer type for the peer-IP limiter used throughout the app.
pub type IpRateLimitLayer = GovernorLayer<PeerIpKeyExtractor, NoOpMiddleware, Body>;

/// Limiter for reads and the dashboard socket, from configuration.
#[must_use]
pub fn api_rate_limit_layer(config: &RateLimitConfig) -> IpRateLimitLayer {
    build(config.replenish_interval_ms(), config.burst_size)
}

fn build(replenish_interval_ms: u64, burst_size: u32) -> IpRateLimitLayer {
    let config = GovernorConfigBuilder::default()
        .per_millisecond(replenish_interval_ms)
        .burst_size(burst_size)
        .finish()
        // The builder only rejects zero values, which configuration
        // validation already refuses.
        .expect("rate limit interval and burst size must be non-zero");

    GovernorLayer::new(Arc::new(config)).error_handler(to_envelope)
}

fn to_envelope(error: GovernorError) -> Response<Body> {
    match error {
        GovernorError::TooManyRequests { wait_time, .. } => {
            AppError::rate_limited(format!("rate limit exceeded; retry in {wait_time}s"))
        }
        GovernorError::UnableToExtractKey => {
            AppError::internal("could not determine the client address for rate limiting")
        }
        GovernorError::Other { msg, .. } => {
            AppError::internal(msg.unwrap_or_else(|| "rate limiter failure".to_owned()))
        }
    }
    .into_response()
}
