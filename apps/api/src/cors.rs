use axum::http::{HeaderName, HeaderValue, Method, header};
use tower_http::cors::CorsLayer;

pub fn build_cors_layer(origins: &[String], _allow_dev_auth: bool) -> CorsLayer {
    cors_from_origins(origins)
}

fn cors_from_origins(origins: &[String]) -> CorsLayer {
    let allowed = origins
        .iter()
        .filter_map(|origin| HeaderValue::from_str(origin).ok())
        .collect::<Vec<_>>();

    CorsLayer::new()
        .allow_origin(allowed)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
            HeaderName::from_static("x-request-id"),
        ])
        .allow_credentials(true)
}
