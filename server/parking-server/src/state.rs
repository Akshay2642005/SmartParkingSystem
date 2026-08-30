use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Instant,
};

use configuration::Config;

use crate::{
    events::{EventReceiver, EventSender, ServerEvent},
    store::SharedStore,
};

#[derive(Debug, Default)]
pub struct HealthFlags {
    mqtt_connected: AtomicBool,
    /// Wall-clock ms of the last accepted device message; 0 means none yet.
    last_ingest_ms: AtomicU64,
}

impl HealthFlags {
    pub fn set_mqtt_connected(&self, connected: bool) {
        self.mqtt_connected.store(connected, Ordering::Relaxed);
    }

    #[must_use]
    pub fn mqtt_connected(&self) -> bool {
        self.mqtt_connected.load(Ordering::Relaxed)
    }

    pub fn record_ingest(&self, at_ms: u64) {
        self.last_ingest_ms.store(at_ms, Ordering::Relaxed);
    }

    #[must_use]
    pub fn last_ingest_ms(&self) -> Option<u64> {
        match self.last_ingest_ms.load(Ordering::Relaxed) {
            0 => None,
            ms => Some(ms),
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub store: SharedStore,
    pub health: Arc<HealthFlags>,
    events: EventSender,
    started_at: Instant,
}

impl AppState {
    pub fn new(config: Arc<Config>, store: SharedStore, events: EventSender) -> Self {
        Self {
            config,
            store,
            health: Arc::new(HealthFlags::default()),
            events,
            started_at: Instant::now(),
        }
    }

    pub fn publish(&self, event: ServerEvent) {
        let _ = self.events.send(event);
    }

    #[must_use]
    pub fn subscribe(&self) -> EventReceiver {
        self.events.subscribe()
    }

    #[must_use]
    pub fn subscriber_count(&self) -> usize {
        self.events.receiver_count()
    }

    #[must_use]
    pub fn uptime_secs(&self) -> u64 {
        self.started_at.elapsed().as_secs()
    }
}
