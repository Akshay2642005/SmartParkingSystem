use axum::http::{
    HeaderValue, Method,
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
};
use configuration::Config;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};

pub fn build_cors_layer(config: &Config) -> CorsLayer {
    let allow_credentials = !config
        .server
        .cors_allowed_origins
        .iter()
        .any(|origin| origin == "*");

    let layer = if allow_credentials {
        let origins = config
            .server
            .cors_allowed_origins
            .iter()
            .filter_map(|origin| HeaderValue::from_str(origin).ok())
            .collect::<Vec<_>>();

        CorsLayer::new()
            .allow_origin(AllowOrigin::list(origins))
            .allow_credentials(true)
    } else {
        CorsLayer::new().allow_origin(Any)
    };

    let layer = layer.allow_methods([
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::PATCH,
        Method::DELETE,
        Method::OPTIONS,
    ]);

    if allow_credentials {
        layer.allow_headers([CONTENT_TYPE, AUTHORIZATION, ACCEPT])
    } else {
        layer.allow_headers(Any)
    }
}
