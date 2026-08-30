use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};
use utoipa::ToSchema;

use crate::protocol::{SectionSnapshot, Slot, TopicKind, ValidationError};

pub type SectionKey = (String, String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct SectionState {
    pub site: String,
    pub section: String,
    pub seq: u64,
    pub slot_count: usize,
    pub slots: Vec<Slot>,
    pub server_ts_ms: u64,
}

impl SectionState {
    #[must_use]
    pub fn key(&self) -> SectionKey {
        (self.site.clone(), self.section.clone())
    }

    #[must_use]
    pub fn free_count(&self) -> usize {
        self.slots
            .iter()
            .filter(|slot| slot.state == crate::protocol::SlotState::Free)
            .count()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectionGuard {
    pub last_seq: u64,
    pub slot_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum NodeStatus {
    Online,
    Offline,
}

impl NodeStatus {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Online => "online",
            Self::Offline => "offline",
        }
    }
}

impl std::fmt::Display for NodeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Reject {
    #[error("invalid MQTT topic: {0}")]
    InvalidTopic(String),

    #[error("malformed JSON: {0}")]
    MalformedJson(String),

    #[error("schema validation failed: {0}")]
    Schema(String),

    #[error("topic/payload section mismatch: topic={topic_section}, payload={payload_section}")]
    TopicMismatch {
        topic_section: String,
        payload_section: String,
    },

    #[error("stale sequence number: seq={seq}, last_seq={last_seq}")]
    Stale { seq: u64, last_seq: u64 },

    #[error("storage failure: {0}")]
    Storage(String),
}

impl From<ValidationError> for Reject {
    fn from(error: ValidationError) -> Self {
        match error {
            ValidationError::MalformedJson(inner) => Self::MalformedJson(inner.to_string()),
            other => Self::Schema(other.to_string()),
        }
    }
}

pub fn parse_topic(topic: &str) -> Result<(&str, &str, TopicKind), Reject> {
    let mut parts = topic.split('/');

    let prefix = parts.next();
    let site = parts.next();
    let section = parts.next();
    let kind = parts.next();
    let suffix = parts.next();

    let known_kind = match (kind, suffix) {
        (Some("state"), None) => Some(TopicKind::State),
        (Some("status"), None) => Some(TopicKind::Status),
        _ => None,
    };

    let (Some(kind), Some(site), Some(section)) = (known_kind, site, section) else {
        return Err(Reject::InvalidTopic(topic.to_string()));
    };

    if prefix != Some(crate::protocol::TOPIC_ROOT) || site.is_empty() || section.is_empty() {
        return Err(Reject::InvalidTopic(topic.to_string()));
    }

    Ok((site, section, kind))
}

pub fn parse_status<'topic>(
    topic: &'topic str,
    payload: &[u8],
) -> Result<(&'topic str, &'topic str, NodeStatus), Reject> {
    let (site, section, kind) = parse_topic(topic)?;

    if kind != TopicKind::Status {
        return Err(Reject::InvalidTopic(topic.to_string()));
    }

    let status = std::str::from_utf8(payload)
        .map_err(|_| Reject::InvalidTopic(topic.to_string()))?
        .trim();

    match status {
        "online" => Ok((site, section, NodeStatus::Online)),
        "offline" => Ok((site, section, NodeStatus::Offline)),
        other => Err(Reject::InvalidTopic(format!("{topic} ({other})"))),
    }
}

pub fn decide(
    topic: &str,
    payload: &[u8],
    guard: Option<SectionGuard>,
) -> Result<SectionState, Reject> {
    let (site, topic_section, kind) = parse_topic(topic)?;

    if kind != TopicKind::State {
        return Err(Reject::InvalidTopic(topic.to_string()));
    }

    let snapshot = SectionSnapshot::parse(payload)?;

    if snapshot.section != topic_section {
        return Err(Reject::TopicMismatch {
            topic_section: topic_section.to_string(),
            payload_section: snapshot.section,
        });
    }

    if let Some(guard) = guard {
        if snapshot.slots.len() != guard.slot_count {
            return Err(Reject::Schema(format!(
                "slot count changed: got {}, section carries {}",
                snapshot.slots.len(),
                guard.slot_count
            )));
        }

        if snapshot.seq < guard.last_seq {
            return Err(Reject::Stale {
                seq: snapshot.seq,
                last_seq: guard.last_seq,
            });
        }
    }

    Ok(SectionState {
        site: site.to_string(),
        section: topic_section.to_string(),
        seq: snapshot.seq,
        slot_count: snapshot.slots.len(),
        slots: snapshot.slots,
        server_ts_ms: now_ms(),
    })
}

/// Wall-clock milliseconds since the UNIX epoch, used for `server_ts_ms`.
#[must_use]
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
