//! Security header middleware builders for HTTP responses.
use axum::http::{
    HeaderValue,
    header::{CACHE_CONTROL, CONTENT_SECURITY_POLICY, HeaderName},
};
use tower_http::set_header::SetResponseHeaderLayer;

/// Builds a cache-control header layer to prevent client caching.
#[must_use]
pub fn cache_control_layer() -> SetResponseHeaderLayer<HeaderValue> {
    SetResponseHeaderLayer::if_not_present(
        CACHE_CONTROL,
        HeaderValue::from_static("no-store, no-cache, must-revalidate"),
    )
}

/// Builds an X-Content-Type-Options header layer to disable sniffing.
#[must_use]
pub fn content_type_options_layer() -> SetResponseHeaderLayer<HeaderValue> {
    SetResponseHeaderLayer::if_not_present(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    )
}

/// Builds an X-Frame-Options header layer to prevent clickjacking.
#[must_use]
pub fn frame_options_layer() -> SetResponseHeaderLayer<HeaderValue> {
    SetResponseHeaderLayer::if_not_present(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    )
}

/// Builds a Referrer-Policy header layer to limit referrer leakage.
#[must_use]
pub fn referrer_policy_layer() -> SetResponseHeaderLayer<HeaderValue> {
    SetResponseHeaderLayer::if_not_present(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("no-referrer"),
    )
}

/// Builds a Content-Security-Policy header layer.
#[must_use]
pub fn csp_layer() -> SetResponseHeaderLayer<HeaderValue> {
    let csp = concat!(
        "default-src 'none'; ",
        "script-src 'self' https://cdn.jsdelivr.net 'unsafe-inline'; ",
        "style-src 'self' https://cdn.jsdelivr.net 'unsafe-inline'; ",
        "img-src 'self' data: https:; ",
        "font-src 'self' https://cdn.jsdelivr.net https://fonts.scalar.com; ",
        "connect-src 'self' https://cdn.jsdelivr.net http://localhost:* https://api.scalar.com; "
    );

    SetResponseHeaderLayer::if_not_present(CONTENT_SECURITY_POLICY, HeaderValue::from_static(csp))
}

/// Builds a Strict-Transport-Security header layer (1 year, includeSubDomains).
#[must_use]
pub fn hsts_layer() -> SetResponseHeaderLayer<HeaderValue> {
    SetResponseHeaderLayer::if_not_present(
        HeaderName::from_static("strict-transport-security"),
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    )
}
