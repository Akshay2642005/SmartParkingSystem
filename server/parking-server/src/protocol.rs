use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub const PROTOCOL_VERSION: u8 = 1;

pub const MAX_SLOTS: usize = 64;

pub const TOPIC_ROOT: &str = "parking";

pub const TOPIC_FILTER: &str = "parking/#";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum SlotState {
    Free,
    Occupied,
    Error,
}

impl SlotState {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Free => "free",
            Self::Occupied => "occupied",
            Self::Error => "error",
        }
    }
}

impl std::fmt::Display for SlotState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for SlotState {
    type Err = UnknownSlotState;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "free" => Ok(Self::Free),
            "occupied" => Ok(Self::Occupied),
            "error" => Ok(Self::Error),
            other => Err(UnknownSlotState(other.to_owned())),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("unknown slot state token: {0}")]
pub struct UnknownSlotState(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct Slot {
    pub id: String,
    pub state: SlotState,
    pub changed_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SectionSnapshot {
    pub v: u8,
    pub ts_ms: u64,
    pub seq: u64,
    pub section: String,
    pub slots: Vec<Slot>,
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("malformed JSON: {0}")]
    MalformedJson(serde_json::Error),

    #[error("schema violation: {0}")]
    Schema(serde_json::Error),

    #[error("unsupported version: {0}")]
    UnsupportedVersion(u8),

    #[error("invalid slot count: {0}")]
    InvalidSlotCount(usize),

    #[error("section name is empty")]
    EmptySection,

    #[error("duplicate slot id: {0}")]
    DuplicateSlotId(String),
}

impl SectionSnapshot {
    pub fn parse(payload: &[u8]) -> Result<Self, ValidationError> {
        let snapshot: Self =
            serde_json::from_slice(payload).map_err(|error| match error.classify() {
                serde_json::error::Category::Data => ValidationError::Schema(error),
                _ => ValidationError::MalformedJson(error),
            })?;

        if snapshot.v != PROTOCOL_VERSION {
            return Err(ValidationError::UnsupportedVersion(snapshot.v));
        }

        if snapshot.slots.is_empty() || snapshot.slots.len() > MAX_SLOTS {
            return Err(ValidationError::InvalidSlotCount(snapshot.slots.len()));
        }

        if snapshot.section.is_empty() {
            return Err(ValidationError::EmptySection);
        }

        let mut seen = std::collections::HashSet::with_capacity(snapshot.slots.len());
        for slot in &snapshot.slots {
            if !seen.insert(slot.id.as_str()) {
                return Err(ValidationError::DuplicateSlotId(slot.id.clone()));
            }
        }

        Ok(snapshot)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopicKind {
    State,
    Status,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payload(slots: serde_json::Value) -> Vec<u8> {
        serde_json::json!({ "v": 1, "ts_ms": 10, "seq": 1, "section": "A", "slots": slots })
            .to_string()
            .into_bytes()
    }

    #[test]
    fn slot_state_round_trips_through_its_protocol_token() {
        for state in [SlotState::Free, SlotState::Occupied, SlotState::Error] {
            assert_eq!(state.as_str().parse::<SlotState>().unwrap(), state);
            assert_eq!(
                serde_json::to_string(&state).unwrap(),
                format!("\"{}\"", state.as_str())
            );
        }

        assert!("unknown".parse::<SlotState>().is_err());
    }

    #[test]
    fn duplicate_slot_ids_are_a_schema_violation() {
        let result = SectionSnapshot::parse(&payload(serde_json::json!([
            { "id": "A-1", "state": "free", "changed_ms": 0 },
            { "id": "A-1", "state": "occupied", "changed_ms": 1 }
        ])));

        assert!(matches!(result, Err(ValidationError::DuplicateSlotId(id)) if id == "A-1"));
    }

    #[test]
    fn slot_count_bound_is_enforced() {
        let slots = (0..=MAX_SLOTS)
            .map(|index| {
                serde_json::json!({ "id": format!("A-{index}"), "state": "free", "changed_ms": 0 })
            })
            .collect::<Vec<_>>();

        assert!(matches!(
            SectionSnapshot::parse(&payload(serde_json::json!(slots))),
            Err(ValidationError::InvalidSlotCount(_))
        ));
    }
}
