//! System service — produces domain types, no HTTP concerns.

use crate::{domain::health::*, state::AppState};
use chrono::Utc;

pub fn livez() -> ProbeResult {
    ProbeResult {
        status: ProbeStatus::Ok,
        timestamp: Utc::now(),
    }
}

pub async fn readyz(state: &AppState) -> ProbeResult {
    let ingest = ingest_status(state);
    let store = store_status(state).await;

    let status = if ingest.health.is_healthy() && store.health.is_healthy() {
        ProbeStatus::Ready
    } else {
        tracing::warn!(
            ingest = ?ingest.detail,
            store = ?store.detail,
            "readiness check failed"
        );
        ProbeStatus::NotReady
    };

    ProbeResult {
        status,
        timestamp: Utc::now(),
    }
}

pub async fn status(state: &AppState) -> StatusResult {
    let config = &state.config;

    let ingest = ingest_status(state);
    let store = store_status(state).await;

    let overall = if ingest.health.is_healthy() && store.health.is_healthy() {
        ComponentHealth::Healthy
    } else {
        ComponentHealth::Unhealthy
    };

    let sections = state
        .store
        .snapshot_all()
        .await
        .map(|sections| sections.len())
        .unwrap_or_default();

    StatusResult {
        overall,
        service: config.primary.name.clone(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        environment: config.primary.env.clone(),
        timestamp: Utc::now(),
        uptime_secs: state.uptime_secs(),
        ingest,
        store,
        sections,
        subscribers: state.subscriber_count(),
    }
}

fn ingest_status(state: &AppState) -> ComponentStatus {
    if state.health.mqtt_connected() {
        ComponentStatus {
            health: ComponentHealth::Healthy,
            detail: state
                .health
                .last_ingest_ms()
                .map(|ms| format!("last accepted message at {ms} ms")),
        }
    } else {
        ComponentStatus {
            health: ComponentHealth::Unhealthy,
            detail: Some(format!(
                "not connected to broker {}",
                state.config.mqtt.broker_uri
            )),
        }
    }
}

async fn store_status(state: &AppState) -> ComponentStatus {
    match state.store.health().await {
        Ok(()) => ComponentStatus {
            health: ComponentHealth::Healthy,
            detail: Some(state.store.backend().to_owned()),
        },
        Err(error) => ComponentStatus {
            health: ComponentHealth::Unhealthy,
            detail: Some(format!("{}: {error}", state.store.backend())),
        },
    }
}
