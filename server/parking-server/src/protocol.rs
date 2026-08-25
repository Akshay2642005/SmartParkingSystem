use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u8 = 1;

/// Sanity cap on slots per section. The real count is a deployment property
/// (the dev board has 3 sensors, target boards 4), so consumers learn it from
/// the first accepted snapshot instead of hardcoding it - see state.rs.
pub const MAX_SLOTS: usize = 64;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SlotState {
    #[serde(rename = "free")]
    Free,

    #[serde(rename = "occupied")]
    Occupied,

    #[serde(rename = "error")]
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Slot {
    pub id: String,
    pub state: SlotState,
    pub changed_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SectionSnapshot {
    pub v: u8,
    #[allow(dead_code)]
    pub ts_ms: u64,
    pub seq: u64,
    pub section: String,
    pub slots: Vec<Slot>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SectionState {
    pub site: String,
    pub section: String,
    pub seq: u64,
    pub slot_count: usize,
    pub slots: Vec<Slot>,
    pub server_ts_ms: u64,
}

#[derive(Debug)]
pub enum ValidationError {
    MalformedJson(serde_json::Error),
    Schema(serde_json::Error),
    UnsupportedVersion(u8),
    InvalidSlotCount(usize),
    EmptySection,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::MalformedJson(e) => write!(f, "Malformed JSON: {}", e),
            ValidationError::Schema(e) => write!(f, "Schema violation: {}", e),
            ValidationError::UnsupportedVersion(v) => write!(f, "Unsupported version: {}", v),
            ValidationError::InvalidSlotCount(count) => write!(f, "Invalid slot count: {}", count),
            ValidationError::EmptySection => write!(f, "Section name is empty"),
        }
    }
}

impl std::error::Error for ValidationError {}

impl SectionSnapshot {
    /// Parse and validate a protocol v1 snapshot. JSON syntax errors are
    /// malformed input; type mismatches (unknown state tokens, wrong field
    /// types, missing required fields such as slots[].id) are schema
    /// violations - communication.md requires rejecting both loudly.
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

        Ok(snapshot)
    }
}
