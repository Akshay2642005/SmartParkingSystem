//! In-memory parking state - the ephemeral backend.
//!
//! Chosen when no `store` is configured. Every guard lives in the map, so a
//! restart forgets both the stale-sequence baseline and the learned slot
//! count; retained broker state repopulates it within one publish cycle.

use async_trait::async_trait;
use std::{
    collections::HashMap,
    sync::{Mutex, MutexGuard},
};

use crate::{
    domain::parking::{
        NodeStatus, Reject, SectionGuard, SectionKey, SectionState, decide, parse_status,
    },
    store::{SectionStore, StatusOutcome},
};

#[derive(Debug, Default)]
pub struct MemoryStore {
    sections: Mutex<HashMap<SectionKey, SectionState>>,
}

impl MemoryStore {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// A poisoned mutex means some other task panicked mid-update. The map is
    /// a cache of device truth that the next retained snapshot restores, so
    /// recovering beats taking the ingest loop down.
    fn lock(&self) -> MutexGuard<'_, HashMap<SectionKey, SectionState>> {
        self.sections.lock().unwrap_or_else(|poisoned| {
            tracing::error!("parking state mutex was poisoned; recovering");
            poisoned.into_inner()
        })
    }
}

#[async_trait]
impl SectionStore for MemoryStore {
    async fn apply_update(&self, topic: &str, payload: &[u8]) -> Result<SectionState, Reject> {
        let mut sections = self.lock();

        let guard = |key: &SectionKey| {
            sections.get(key).map(|state| SectionGuard {
                last_seq: state.seq,
                slot_count: state.slot_count,
            })
        };

        // The key is only known after the topic parses, which `decide` does;
        // parse it here too so the guard lookup can happen first.
        let (site, section, _) = crate::domain::parking::parse_topic(topic)?;
        let key = (site.to_string(), section.to_string());

        let state = decide(topic, payload, guard(&key))?;

        sections.insert(key, state.clone());

        Ok(state)
    }

    async fn apply_status(&self, topic: &str, payload: &[u8]) -> Result<StatusOutcome, Reject> {
        let (site, section, status) = parse_status(topic, payload)?;

        if status == NodeStatus::Offline {
            self.lock().remove(&(site.to_string(), section.to_string()));
        }

        Ok(StatusOutcome {
            site: site.to_string(),
            section: section.to_string(),
            status,
        })
    }

    async fn snapshot_all(&self) -> Result<Vec<SectionState>, Reject> {
        Ok(self.lock().values().cloned().collect())
    }

    async fn get(&self, site: &str, section: &str) -> Result<Option<SectionState>, Reject> {
        Ok(self
            .lock()
            .get(&(site.to_string(), section.to_string()))
            .cloned())
    }

    fn backend(&self) -> &'static str {
        "memory"
    }

    async fn health(&self) -> Result<(), String> {
        Ok(())
    }
}
