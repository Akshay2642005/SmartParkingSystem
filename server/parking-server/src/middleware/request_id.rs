use axum::http::{
    HeaderMap,
    header::{HeaderName, HeaderValue},
};

#[must_use]
pub fn request_id_header_name() -> HeaderName {
    HeaderName::from_static("x-request-id")
}

#[must_use]
pub fn request_id_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get(request_id_header_name())
        .and_then(|value: &HeaderValue| value.to_str().ok())
        .map(ToOwned::to_owned)
}
