use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeStatus {
    Ok,
    Ready,
    NotReady,
}

impl ProbeStatus {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Ready => "ready",
            Self::NotReady => "not ready",
        }
    }
    #[must_use]
    pub fn is_ready(&self) -> bool {
        !matches!(self, Self::NotReady)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentHealth {
    Healthy,
    Unhealthy,
}

impl ComponentHealth {
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        matches!(self, Self::Healthy)
    }
}

#[derive(Debug, Clone)]
pub struct ProbeResult {
    pub status: ProbeStatus,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ComponentStatus {
    pub health: ComponentHealth,
    pub detail: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StatusResult {
    pub overall: ComponentHealth,
    pub service: String,
    pub version: String,
    pub environment: String,
    pub timestamp: DateTime<Utc>,
    pub uptime_secs: u64,
    /// Device feed: the MQTT subscription this service ingests from.
    pub ingest: ComponentStatus,
    /// Parking state backend (`memory` or `postgres`).
    pub store: ComponentStatus,
    /// Sections currently known, and dashboard sockets attached.
    pub sections: usize,
    pub subscribers: usize,
}
