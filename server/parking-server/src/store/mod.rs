#![allow(dead_code, unused)]
mod memory;

use crate::domain::parking::{NodeStatus, Reject, SectionState};
use async_trait::async_trait;
pub use memory::MemoryStore;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusOutcome {
    pub site: String,
    pub section: String,
    pub status: NodeStatus,
}

#[async_trait]
pub trait SectionStore: Send + Sync + 'static {
    async fn apply_update(&self, topic: &str, payload: &[u8]) -> Result<SectionState, Reject>;
    async fn apply_status(&self, topic: &str, payload: &[u8]) -> Result<StatusOutcome, Reject>;
    async fn snapshot_all(&self) -> Result<Vec<SectionState>, Reject>;
    async fn get(&self, site: &str, section: &str) -> Result<Option<SectionState>, Reject>;
    fn backend(&self) -> &'static str;
    async fn health(&self) -> Result<(), String>;
}

pub type SharedStore = Arc<dyn SectionStore>;
