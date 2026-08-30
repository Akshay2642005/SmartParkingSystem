use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    Internal,
    NotFound,
    BadRequest,
    RequestTimeout,
    RateLimited,
    Unauthorized,
    ServiceUnavailable,
    CommandsNotSupported,
    InvalidFrame,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorBody {
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

#[derive(Debug)]
pub struct AppError {
    pub status: StatusCode,
    pub code: ErrorCode,
    pub message: String,
    pub request_id: Option<String>,
}

impl AppError {
    #[must_use]
    pub fn new(status: StatusCode, code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
            request_id: None,
        }
    }

    #[must_use]
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, ErrorCode::NotFound, message)
    }

    #[must_use]
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::Internal,
            message,
        )
    }

    #[must_use]
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, ErrorCode::Unauthorized, message)
    }

    #[must_use]
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, ErrorCode::BadRequest, message)
    }

    #[must_use]
    pub fn request_timeout(message: impl Into<String>) -> Self {
        Self::new(
            StatusCode::REQUEST_TIMEOUT,
            ErrorCode::RequestTimeout,
            message,
        )
    }

    #[must_use]
    pub fn rate_limited(message: impl Into<String>) -> Self {
        Self::new(
            StatusCode::TOO_MANY_REQUESTS,
            ErrorCode::RateLimited,
            message,
        )
    }

    #[must_use]
    pub fn service_unavailable(message: impl Into<String>) -> Self {
        Self::new(
            StatusCode::SERVICE_UNAVAILABLE,
            ErrorCode::ServiceUnavailable,
            message,
        )
    }

    /// Attach the request id so the envelope can be correlated with logs.
    #[must_use]
    pub fn with_request_id(mut self, request_id: Option<String>) -> Self {
        self.request_id = request_id;
        self
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = Json(ErrorResponse {
            error: ErrorBody {
                code: self.code,
                message: self.message,
                request_id: self.request_id,
            },
        });

        (self.status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;

    async fn body_of(error: AppError) -> (StatusCode, serde_json::Value) {
        let response = error.into_response();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();

        (status, serde_json::from_slice(&bytes).unwrap())
    }

    #[tokio::test]
    async fn envelope_nests_a_machine_readable_code() {
        let (status, json) = body_of(AppError::not_found("unknown section")).await;

        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(json["error"]["code"], "not_found");
        assert_eq!(json["error"]["message"], "unknown section");
        assert!(
            json["error"].get("request_id").is_none(),
            "absent request id must be omitted, not null"
        );
    }

    #[tokio::test]
    async fn request_id_is_echoed_when_known() {
        let (status, json) = body_of(
            AppError::request_timeout("request timed out").with_request_id(Some("abc-123".into())),
        )
        .await;

        assert_eq!(status, StatusCode::REQUEST_TIMEOUT);
        assert_eq!(json["error"]["code"], "request_timeout");
        assert_eq!(json["error"]["request_id"], "abc-123");
    }

    #[test]
    fn codes_serialize_as_snake_case_contract_strings() {
        let codes = [
            (ErrorCode::Internal, "internal"),
            (ErrorCode::NotFound, "not_found"),
            (ErrorCode::BadRequest, "bad_request"),
            (ErrorCode::RequestTimeout, "request_timeout"),
            (ErrorCode::RateLimited, "rate_limited"),
            (ErrorCode::Unauthorized, "unauthorized"),
            (ErrorCode::ServiceUnavailable, "service_unavailable"),
            (ErrorCode::CommandsNotSupported, "commands_not_supported"),
            (ErrorCode::InvalidFrame, "invalid_frame"),
        ];

        for (code, expected) in codes {
            assert_eq!(serde_json::to_value(code).unwrap(), expected);
        }
    }
}
