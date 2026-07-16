use crate::services::metrics::record_http_request;
use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use std::time::Instant;

pub async fn track_metrics(request: Request, next: Next) -> Response {
    let method = request.method().to_string();
    let path = normalize_path(request.uri().path());
    let started = Instant::now();
    let response = next.run(request).await;
    let status = response.status().as_u16();
    record_http_request(&method, &path, status, started.elapsed().as_secs_f64());
    response
}

fn normalize_path(path: &str) -> String {
    // Avoid high-cardinality labels from UUIDs / hashes.
    path.split('/')
        .map(|segment| {
            if looks_like_id(segment) {
                ":id"
            } else {
                segment
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn looks_like_id(segment: &str) -> bool {
    if segment.is_empty() {
        return false;
    }
    if segment.starts_with("0x") && segment.len() >= 10 {
        return true;
    }
    uuid::Uuid::parse_str(segment).is_ok()
}
