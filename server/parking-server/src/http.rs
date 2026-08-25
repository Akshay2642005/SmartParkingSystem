use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::get};

use crate::state::SharedStateStore;

pub fn router(store: SharedStateStore) -> Router {
    Router::new()
        .route("/internal/state", get(get_state))
        .with_state(store)
}

async fn get_state(State(store): State<SharedStateStore>) -> impl IntoResponse {
    let guard = match store.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let state = guard.snapshot();

    tracing::info!(count = state.len(), ?state, "returning internal state");

    (StatusCode::OK, Json(state))
}
