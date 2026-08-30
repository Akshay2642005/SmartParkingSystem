use crate::domain::health;
use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ProbeResponse {
    pub status: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ComponentStatus {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct StatusChecks {
    /// Device feed (MQTT subscription).
    pub ingest: ComponentStatus,
    /// Parking state backend (`memory` or `postgres`).
    pub store: ComponentStatus,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct StatusResponse {
    pub status: String,
    pub service: String,
    pub version: String,
    pub environment: String,
    pub timestamp: DateTime<Utc>,
    pub uptime_secs: u64,
    pub sections: usize,
    pub subscribers: usize,
    pub checks: StatusChecks,
}

impl From<health::ProbeResult> for ProbeResponse {
    fn from(result: health::ProbeResult) -> Self {
        Self {
            status: result.status.as_str().to_owned(),
            timestamp: result.timestamp,
        }
    }
}

impl From<health::ComponentStatus> for ComponentStatus {
    fn from(status: health::ComponentStatus) -> Self {
        Self {
            status: if status.health.is_healthy() {
                "healthy".to_owned()
            } else {
                "unhealthy".to_owned()
            },
            detail: status.detail,
        }
    }
}

impl From<health::StatusResult> for StatusResponse {
    fn from(result: health::StatusResult) -> Self {
        Self {
            status: if result.overall.is_healthy() {
                "healthy".to_owned()
            } else {
                "degraded".to_owned()
            },
            service: result.service,
            version: result.version,
            environment: result.environment,
            timestamp: result.timestamp,
            uptime_secs: result.uptime_secs,
            sections: result.sections,
            subscribers: result.subscribers,
            checks: StatusChecks {
                ingest: result.ingest.into(),
                store: result.store.into(),
            },
        }
    }
}
