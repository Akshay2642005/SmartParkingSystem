mod cors;
mod rate_limit;
mod request_id;
mod security;
mod timeout;

pub use rate_limit::api_rate_limit_layer;

use axum::{
    Router,
    extract::DefaultBodyLimit,
    http::{Request, Response},
};
use configuration::Config;
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    limit::RequestBodyLimitLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    sensitive_headers::SetSensitiveRequestHeadersLayer,
};

/// Applies standard middleware layers to the provided router.
///
/// Uses configuration to derive CORS, limits, and timeouts.
pub fn apply(router: Router, config: &Config) -> Router {
    let request_id_header = request_id::request_id_header_name();
    let cors_layer = cors::build_cors_layer(config);
    let max_body_size = config.server.max_body_size_bytes;
    let request_timeout_secs = config.server.request_timeout_secs;

    let service_stack = ServiceBuilder::new()
        // A panicking handler becomes a 500 in the shared envelope and the
        // process keeps serving (release profile keeps unwinding for this).
        .layer(CatchPanicLayer::custom(
            |panic: Box<dyn std::any::Any + Send>| {
                let detail = panic
                    .downcast_ref::<&str>()
                    .map(|message| (*message).to_owned())
                    .or_else(|| panic.downcast_ref::<String>().cloned())
                    .unwrap_or_else(|| "unknown panic".to_owned());
                tracing::error!(panic = %detail, "handler panicked");

                axum::response::IntoResponse::into_response(
                    crate::response::error::AppError::internal("internal server error"),
                )
            },
        ))
        // Order matters: the id must be set on the request (outer) before the
        // propagate layer (inner) can copy it onto the response.
        .layer(SetRequestIdLayer::new(
            request_id_header.clone(),
            MakeRequestUuid,
        ))
        .layer(PropagateRequestIdLayer::new(request_id_header))
        .layer(security::cache_control_layer())
        .layer(security::content_type_options_layer())
        .layer(security::frame_options_layer())
        .layer(security::referrer_policy_layer())
        .layer(security::csp_layer())
        .layer(security::hsts_layer())
        .layer(SetSensitiveRequestHeadersLayer::new([
            axum::http::header::AUTHORIZATION,
            axum::http::header::COOKIE,
        ]))
        .layer(RequestBodyLimitLayer::new(max_body_size))
        .layer(
            tower_http::trace::TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    let request_id = request_id::request_id_from_headers(request.headers())
                        .unwrap_or_else(|| "unknown".to_string());

                    tracing::info_span!(
                        "http_request",
                        request_id = %request_id,
                        method = %request.method(),
                        path = %request.uri().path(),
                        status = tracing::field::Empty,
                        user_id = tracing::field::Empty,
                    )
                })
                .on_response(
                    |response: &Response<_>, latency: Duration, span: &tracing::Span| {
                        let status = response.status();
                        span.record("status", status.as_u16());

                        if status.is_server_error() {
                            tracing::error!(
                                parent: span,
                                status = status.as_u16(),
                                latency_ms = latency.as_millis() as u64,
                                "request completed with server error"
                            );
                        } else if status.is_client_error() {
                            tracing::warn!(
                                parent: span,
                                status = status.as_u16(),
                                latency_ms = latency.as_millis() as u64,
                                "request completed with client error"
                            );
                        } else {
                            tracing::info!(
                                parent: span,
                                status = status.as_u16(),
                                latency_ms = latency.as_millis() as u64,
                                "request completed"
                            );
                        }
                    },
                ),
        )
        .layer(cors_layer)
        .layer(CompressionLayer::new());

    router
        .layer(DefaultBodyLimit::max(max_body_size))
        .route_layer(axum::middleware::from_fn(move |request, next| {
            timeout::timeout_middleware(request, next, request_timeout_secs)
        }))
        .layer(service_stack)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode},
        routing::{get, post},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    /// Router with routes that exercise the failure paths the stack must own.
    fn app(config: &Config) -> Router {
        let router = Router::new()
            .route(
                "/slow",
                get(|| async {
                    tokio::time::sleep(Duration::from_secs(30)).await;
                    "never"
                }),
            )
            .route(
                "/panic",
                get(|| async {
                    panic!("handler exploded on purpose");
                    #[allow(unreachable_code)]
                    "never"
                }),
            )
            .route("/echo", post(|body: String| async move { body }));

        apply(router, config)
    }

    async fn json_of(response: axum::response::Response) -> serde_json::Value {
        let bytes = response.into_body().collect().await.unwrap().to_bytes();

        serde_json::from_slice(&bytes).expect("failures must return the JSON envelope")
    }

    #[tokio::test]
    async fn a_timed_out_request_returns_the_envelope_with_its_request_id() {
        let mut config = Config::default();
        config.server.request_timeout_secs = 1;

        let response = app(&config)
            .oneshot(
                Request::builder()
                    .uri("/slow")
                    .header("x-request-id", "drill-42")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
        assert_eq!(response.headers()["x-request-id"], "drill-42");

        let body = json_of(response).await;
        assert_eq!(body["error"]["code"], "request_timeout");
        assert_eq!(body["error"]["request_id"], "drill-42");
    }

    #[tokio::test]
    async fn a_panicking_handler_becomes_a_500_envelope() {
        let config = Config::default();

        let response = app(&config)
            .oneshot(
                Request::builder()
                    .uri("/panic")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(json_of(response).await["error"]["code"], "internal");
    }

    #[tokio::test]
    async fn oversized_bodies_are_refused() {
        let mut config = Config::default();
        config.server.max_body_size_bytes = 16;

        let response = app(&config)
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/echo")
                    .body(Body::from("x".repeat(1024)))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[tokio::test]
    async fn configured_dashboard_origin_is_allowed_by_cors() {
        let mut config = Config::default();
        config.server.cors_allowed_origins = vec!["http://localhost:3000".to_owned()];

        let response = app(&config)
            .oneshot(
                Request::builder()
                    .method(Method::OPTIONS)
                    .uri("/echo")
                    .header("origin", "http://localhost:3000")
                    .header("access-control-request-method", "POST")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.headers()["access-control-allow-origin"],
            "http://localhost:3000"
        );
    }
}
