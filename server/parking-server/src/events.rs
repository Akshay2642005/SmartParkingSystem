use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use utoipa::ToSchema;

use crate::{
    domain::parking::{NodeStatus, SectionState, now_ms},
    response::error::ErrorCode,
};

pub const CHANNEL_CAPACITY: usize = 256;

pub type EventSender = broadcast::Sender<ServerEvent>;
pub type EventReceiver = broadcast::Receiver<ServerEvent>;

#[must_use]
pub fn channel() -> EventSender {
    broadcast::Sender::new(CHANNEL_CAPACITY)
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerEvent {
    Snapshot {
        sections: Vec<SectionState>,
        server_ts_ms: u64,
    },
    Update {
        section: SectionState,
    },
    NodeStatus {
        site: String,
        section: String,
        status: NodeStatus,
        server_ts_ms: u64,
    },
    Error {
        code: ErrorCode,
        message: String,
    },
}

impl ServerEvent {
    #[must_use]
    pub fn snapshot(sections: Vec<SectionState>) -> Self {
        Self::Snapshot {
            sections,
            server_ts_ms: now_ms(),
        }
    }

    #[must_use]
    pub fn update(section: SectionState) -> Self {
        Self::Update { section }
    }

    #[must_use]
    pub fn node_status(site: String, section: String, status: NodeStatus) -> Self {
        Self::NodeStatus {
            site,
            section,
            status,
            server_ts_ms: now_ms(),
        }
    }

    #[must_use]
    pub fn error(code: ErrorCode, message: impl Into<String>) -> Self {
        Self::Error {
            code,
            message: message.into(),
        }
    }

    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|error| {
            tracing::error!(%error, "failed to serailize server event");
            r#"{"type":"error","code":"internal","message":"event serialization failed"}"#
                .to_owned()
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientFrame {
    Cmd {
        #[serde(default)]
        name: Option<String>,
    },
}

#[must_use]
pub fn reply_to_client_frame(text: &str) -> ServerEvent {
    match serde_json::from_str::<ClientFrame>(text) {
        Ok(ClientFrame::Cmd { name }) => ServerEvent::error(
            ErrorCode::CommandsNotSupported,
            match name {
                Some(name) => format!("command {name:?} is reserved but not supported in v1"),
                None => "commands are reserved but not supported in v1".to_owned(),
            },
        ),
        Err(_) => ServerEvent::error(
            ErrorCode::InvalidFrame,
            "expected a JSON object with a known \"type\"",
        ),
    }
}
