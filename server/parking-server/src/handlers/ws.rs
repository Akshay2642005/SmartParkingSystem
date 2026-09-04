use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
};
use tokio::sync::broadcast::error::RecvError;

use crate::{
    events::{ServerEvent, reply_to_client_frame},
    state::AppState,
};

pub async fn ws_entry(State(state): State<AppState>, upgrade: WebSocketUpgrade) -> Response {
    upgrade.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let mut events = state.subscribe();

    let sections = match state.store.snapshot_all().await {
        Ok(sections) => sections,
        Err(error) => {
            tracing::error!(%error, "failed to read parking state for websocket snapshot");
            let _ = socket
                .send(Message::Text(
                    ServerEvent::error(
                        crate::response::error::ErrorCode::ServiceUnavailable,
                        "parking state is temporarily unavailable",
                    )
                    .to_json()
                    .into(),
                ))
                .await;
            return;
        }
    };

    if socket
        .send(Message::Text(
            ServerEvent::snapshot(sections).to_json().into(),
        ))
        .await
        .is_err()
    {
        return;
    }

    tracing::info!(
        subscribers = state.subscriber_count(),
        "dashboard connected"
    );

    loop {
        tokio::select! {
            event = events.recv() => match event {
                Ok(event) => {
                    if socket.send(Message::Text(event.to_json().into())).await.is_err() {
                        break;
                    }
                }
                // The client fell behind by more than the channel capacity.
                // Closing is the recovery path: it reconnects and re-syncs
                // from a fresh snapshot.
                Err(RecvError::Lagged(missed)) => {
                    tracing::warn!(missed, "dashboard socket lagged; closing so it re-syncs");
                    break;
                }
                Err(RecvError::Closed) => break,
            },

            frame = socket.recv() => match frame {
                Some(Ok(Message::Text(text))) => {
                    let reply = reply_to_client_frame(&text);
                    // Contract: reply with a typed error, keep the socket open.
                    if socket.send(Message::Text(reply.to_json().into())).await.is_err() {
                        break;
                    }
                }
                Some(Ok(Message::Close(_))) | None => break,
                Some(Ok(_)) => {}
                Some(Err(error)) => {
                    tracing::debug!(%error, "dashboard socket error");
                    break;
                }
            },
        }
    }

    tracing::info!("dashboard disconnected");
}
