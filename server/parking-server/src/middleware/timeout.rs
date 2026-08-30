use std::time::Duration;

use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use tokio::time::timeout;

use crate::{middleware::request_id, response::error::AppError};

pub async fn timeout_middleware(
    request: Request,
    next: Next,
    request_timeout_secs: u64,
) -> Response {
    let request_id = request_id::request_id_from_headers(request.headers());
    let path = request.uri().path().to_owned();

    match timeout(Duration::from_secs(request_timeout_secs), next.run(request)).await {
        Ok(response) => response,
        Err(_) => {
            tracing::warn!(path, request_timeout_secs, "request timed out");

            AppError::request_timeout(format!(
                "request exceeded the {request_timeout_secs}s timeout"
            ))
            .with_request_id(request_id)
            .into_response()
        }
    }
}
