use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::protocol::{SectionSnapshot, SectionState};

pub type StateKey = (String, String);

pub type SharedStateStore = Arc<Mutex<StateStore>>;

#[derive(Debug, Default)]
pub struct StateStore {
    sections: HashMap<StateKey, SectionState>,
}

#[derive(Debug)]
pub enum Reject {
    InvalidTopic(String),
    MalformedJson(String),
    Schema(String),
    TopicMismatch {
        topic_section: String,
        payload_section: String,
    },
    Stale {
        seq: u64,
        last_seq: u64,
    },
}

impl std::fmt::Display for Reject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidTopic(topic) => {
                write!(f, "invalid MQTT topic: {topic}")
            }

            Self::MalformedJson(error) => {
                write!(f, "malformed JSON: {error}")
            }

            Self::Schema(error) => {
                write!(f, "schema validation failed: {error}")
            }

            Self::TopicMismatch {
                topic_section,
                payload_section,
            } => {
                write!(
                    f,
                    "topic/payload section mismatch: topic={topic_section}, payload={payload_section}"
                )
            }

            Self::Stale { seq, last_seq } => {
                write!(f, "stale sequence number: seq={seq}, last_seq={last_seq}")
            }
        }
    }
}

impl std::error::Error for Reject {}

pub fn new_shared_store() -> SharedStateStore {
    Arc::new(Mutex::new(StateStore::default()))
}

/// Apply a `parking/{site}/{section}/state` publish to the store.
///
/// The slot count per section is a deployment property (dev node: 3 sensors,
/// target boards: 4), so it is learned from the first accepted snapshot and
/// enforced afterwards - communication.md only promises complete snapshots.
pub fn apply_update(
    store: &mut StateStore,
    topic: &str,
    payload: &[u8],
) -> Result<SectionState, Reject> {
    let (site, topic_section, kind) = parse_topic(topic)?;

    if !matches!(kind, TopicKind::State) {
        return Err(Reject::InvalidTopic(topic.to_string()));
    }

    let snapshot = SectionSnapshot::parse(payload).map_err(|error| match error {
        crate::protocol::ValidationError::MalformedJson(error) => {
            Reject::MalformedJson(error.to_string())
        }

        other => Reject::Schema(other.to_string()),
    })?;

    if snapshot.section != topic_section {
        return Err(Reject::TopicMismatch {
            topic_section: topic_section.to_string(),
            payload_section: snapshot.section,
        });
    }

    let key = (site.to_string(), topic_section.to_string());

    if let Some(previous) = store.sections.get(&key) {
        if snapshot.slots.len() != previous.slots.len() {
            return Err(Reject::Schema(format!(
                "slot count changed: got {}, section carries {}",
                snapshot.slots.len(),
                previous.slots.len()
            )));
        }

        if snapshot.seq < previous.seq {
            return Err(Reject::Stale {
                seq: snapshot.seq,
                last_seq: previous.seq,
            });
        }
    }

    // seq == last_seq is the QoS-1 redelivery case: applying an identical
    // snapshot twice is harmless by design (contract § Idempotency).
    let state = SectionState {
        site: site.to_string(),
        section: topic_section.to_string(),
        seq: snapshot.seq,
        slot_count: snapshot.slots.len(),
        slots: snapshot.slots,
        server_ts_ms: now_ms(),
    };

    store.sections.insert(key, state.clone());

    Ok(state)
}

/// Apply a retained `online|offline` status publish. A node going offline
/// ends its session: its seq counter restarts at 1 on reboot, so the stale
/// baseline must be cleared or every post-reboot snapshot would be rejected
/// forever (contract § Connection Lifecycle - liveness exists for this).
pub fn apply_status(
    store: &mut StateStore,
    topic: &str,
    payload: &[u8],
) -> Result<Option<String>, Reject> {
    let (site, section, kind) = parse_topic(topic)?;

    if !matches!(kind, TopicKind::Status) {
        return Err(Reject::InvalidTopic(topic.to_string()));
    }

    let status = std::str::from_utf8(payload)
        .map_err(|_| Reject::InvalidTopic(topic.to_string()))?
        .trim()
        .to_owned();

    match status.as_str() {
        "offline" => {
            store
                .sections
                .remove(&(site.to_string(), section.to_string()));
            Ok(Some(status))
        }

        "online" => Ok(Some(status)),

        other => Err(Reject::InvalidTopic(format!("{topic} ({other})"))),
    }
}

pub enum TopicKind {
    State,
    Status,
}

/// Validate `parking/{site}/{section}/{kind}` and split it. Identity comes
/// from the topic only - a payload body claiming another section is a
/// hard reject (apply_update).
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

    if prefix != Some("parking") || known_kind.is_none() {
        return Err(Reject::InvalidTopic(topic.to_string()));
    }

    let site = site.ok_or_else(|| Reject::InvalidTopic(topic.to_string()))?;
    let section = section.ok_or_else(|| Reject::InvalidTopic(topic.to_string()))?;

    if site.is_empty() || section.is_empty() {
        return Err(Reject::InvalidTopic(topic.to_string()));
    }

    Ok((site, section, known_kind.unwrap()))
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before UNIX epoch")
        .as_millis() as u64
}

impl StateStore {
    pub fn snapshot(&self) -> Vec<SectionState> {
        self.sections.values().cloned().collect()
    }

    #[allow(unused)]
    pub fn get(&self, site: &str, section: &str) -> Option<SectionState> {
        self.sections
            .get(&(site.to_string(), section.to_string()))
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Contract-shaped snapshot: ids carry the section prefix, changed_ms is
    /// required, states come from free|occupied|error only.
    fn valid_payload(seq: u64, section: &str) -> Vec<u8> {
        serde_json::json!({
            "v": 1,
            "ts_ms": 10,
            "seq": seq,
            "section": section,
            "slots": [
                { "id": "A-1", "state": "free",     "changed_ms": 0 },
                { "id": "A-2", "state": "occupied", "changed_ms": 5 },
                { "id": "A-3", "state": "error",    "changed_ms": 7 }
            ]
        })
        .to_string()
        .into_bytes()
    }

    #[test]
    fn valid_v1_snapshot_advances_sequence() {
        let mut store = StateStore::default();

        let state =
            apply_update(&mut store, "parking/main/A/state", &valid_payload(1, "A")).unwrap();

        assert_eq!(state.site, "main");
        assert_eq!(state.section, "A");
        assert_eq!(state.seq, 1);
        assert_eq!(state.slot_count, 3);
        assert_eq!(state.slots.len(), 3);
    }

    #[test]
    fn duplicate_sequence_reapplies_snapshot_idempotently() {
        let mut store = StateStore::default();

        let first =
            apply_update(&mut store, "parking/main/A/state", &valid_payload(1, "A")).unwrap();

        let duplicate =
            apply_update(&mut store, "parking/main/A/state", &valid_payload(1, "A")).unwrap();

        // QoS 1 redelivery: same seq, harmless to reapply (contract § Idempotency).
        assert_eq!(first.seq, duplicate.seq);
        assert_eq!(first.slots, duplicate.slots);
    }

    #[test]
    fn older_sequence_is_rejected() {
        let mut store = StateStore::default();

        apply_update(&mut store, "parking/main/A/state", &valid_payload(2, "A")).unwrap();

        let result = apply_update(&mut store, "parking/main/A/state", &valid_payload(1, "A"));

        assert!(matches!(
            result,
            Err(Reject::Stale {
                seq: 1,
                last_seq: 2
            })
        ));
    }

    #[test]
    fn offline_status_resets_stale_baseline_after_reboot() {
        let mut store = StateStore::default();

        apply_update(&mut store, "parking/main/A/state", &valid_payload(8, "A")).unwrap();

        // Device reboots; broker delivers its retained LWT...
        let status = apply_status(&mut store, "parking/main/A/status", b"offline").unwrap();
        assert_eq!(status.as_deref(), Some("offline"));

        // ...and the fresh session starts at seq 1 again - must be accepted.
        let state =
            apply_update(&mut store, "parking/main/A/state", &valid_payload(1, "A")).unwrap();

        assert_eq!(state.seq, 1);
    }

    #[test]
    fn online_status_is_accepted_without_touching_state() {
        let mut store = StateStore::default();

        apply_update(&mut store, "parking/main/A/state", &valid_payload(8, "A")).unwrap();

        apply_status(&mut store, "parking/main/A/status", b"online").unwrap();

        let result = apply_update(&mut store, "parking/main/A/state", &valid_payload(1, "A"));

        // No offline in between: the stale guard stays armed.
        assert!(matches!(
            result,
            Err(Reject::Stale {
                seq: 1,
                last_seq: 8
            })
        ));
    }

    #[test]
    fn invalid_status_payload_is_rejected() {
        let mut store = StateStore::default();

        let result = apply_status(&mut store, "parking/main/A/status", b"away");

        assert!(result.is_err());
    }

    #[test]
    fn slot_count_is_learned_then_enforced() {
        let mut store = StateStore::default();

        // Dev node publishes 3 slots - accepted and remembered.
        apply_update(&mut store, "parking/main/A/state", &valid_payload(1, "A")).unwrap();

        let four_slots = serde_json::json!({
            "v": 1,
            "ts_ms": 10,
            "seq": 2,
            "section": "A",
            "slots": [
                { "id": "A-1", "state": "free", "changed_ms": 0 },
                { "id": "A-2", "state": "free", "changed_ms": 0 },
                { "id": "A-3", "state": "free", "changed_ms": 0 },
                { "id": "A-4", "state": "free", "changed_ms": 0 }
            ]
        });

        let result = apply_update(
            &mut store,
            "parking/main/A/state",
            four_slots.to_string().as_bytes(),
        );

        assert!(matches!(result, Err(Reject::Schema(_))));
    }

    #[test]
    fn empty_slot_list_is_rejected() {
        let mut store = StateStore::default();

        let payload = serde_json::json!({
            "v": 1,
            "ts_ms": 10,
            "seq": 1,
            "section": "A",
            "slots": []
        });

        let result = apply_update(
            &mut store,
            "parking/main/A/state",
            payload.to_string().as_bytes(),
        );

        assert!(matches!(result, Err(Reject::Schema(_))));
    }

    #[test]
    fn missing_required_slot_fields_are_schema_errors() {
        let mut store = StateStore::default();

        // No id / changed_ms on slots - contract requires them.
        let payload = serde_json::json!({
            "v": 1,
            "ts_ms": 10,
            "seq": 1,
            "section": "A",
            "slots": [
                { "state": "free" },
                { "state": "occupied" },
                { "state": "error" }
            ]
        });

        let result = apply_update(
            &mut store,
            "parking/main/A/state",
            payload.to_string().as_bytes(),
        );

        assert!(matches!(result, Err(Reject::Schema(_))));
    }

    #[test]
    fn unknown_slot_state_is_a_schema_error() {
        let mut store = StateStore::default();

        // "unknown" left the wire vocabulary in Phase 21; serde must reject.
        let payload = serde_json::json!({
            "v": 1,
            "ts_ms": 10,
            "seq": 1,
            "section": "A",
            "slots": [
                { "id": "A-1", "state": "free",     "changed_ms": 0 },
                { "id": "A-2", "state": "occupied", "changed_ms": 0 },
                { "id": "A-3", "state": "broken",   "changed_ms": 0 }
            ]
        });

        let result = apply_update(
            &mut store,
            "parking/main/A/state",
            payload.to_string().as_bytes(),
        );

        assert!(matches!(result, Err(Reject::Schema(_))));
    }

    #[test]
    fn topic_and_payload_section_must_match() {
        let mut store = StateStore::default();

        let result = apply_update(&mut store, "parking/main/A/state", &valid_payload(1, "B"));

        assert!(matches!(
            result,
            Err(Reject::TopicMismatch {
                topic_section,
                payload_section
            }) if topic_section == "A" && payload_section == "B"
        ));
    }

    #[test]
    fn malformed_json_is_rejected_without_panicking() {
        let mut store = StateStore::default();

        let result = apply_update(&mut store, "parking/main/A/state", b"{invalid");

        assert!(matches!(result, Err(Reject::MalformedJson(_))));
    }

    #[test]
    fn unsupported_version_is_rejected() {
        let mut store = StateStore::default();

        let payload = serde_json::json!({
            "v": 2,
            "ts_ms": 10,
            "seq": 1,
            "section": "A",
            "slots": [
                { "id": "A-1", "state": "free", "changed_ms": 0 }
            ]
        });

        let result = apply_update(
            &mut store,
            "parking/main/A/state",
            payload.to_string().as_bytes(),
        );

        assert!(matches!(result, Err(Reject::Schema(_))));
    }

    #[test]
    fn non_state_topics_are_invalid() {
        let mut store = StateStore::default();

        assert!(apply_update(&mut store, "garbage/topic/here", b"{}").is_err());
        assert!(apply_update(&mut store, "parking//B/state", b"{}").is_err());

        assert!(apply_status(&mut store, "parking/main/A/state", b"online").is_err());
    }
}
