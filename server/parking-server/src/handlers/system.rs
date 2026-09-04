//! HTTP system handlers — convert domain types to HTTP responses.
//!
//! These are mounted unprefixed by `app::ops_router`: probes must stay outside
//! the API prefix and outside the rate limiter, so their mounting is explicit.

use crate::{response::system::*, services::system as svc, state::AppState};
use axum::{Json, extract::State, http::StatusCode};

pub async fn health() -> Json<ProbeResponse> {
    Json(svc::livez().into())
}

pub async fn healthz() -> Json<ProbeResponse> {
    Json(svc::livez().into())
}

pub async fn livez() -> Json<ProbeResponse> {
    Json(svc::livez().into())
}

pub async fn readyz(State(state): State<AppState>) -> (StatusCode, Json<ProbeResponse>) {
    let result = svc::readyz(&state).await;
    let status = if result.status.is_ready() {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status, Json(result.into()))
}

pub async fn status(State(state): State<AppState>) -> (StatusCode, Json<StatusResponse>) {
    let result = svc::status(&state).await;
    let status = if result.overall.is_healthy() {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status, Json(result.into()))
}
