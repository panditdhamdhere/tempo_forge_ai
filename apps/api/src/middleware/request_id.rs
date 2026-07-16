use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

pub async fn attach_request_id(mut request: Request, next: Next) -> Response {
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    request.headers_mut().insert(
        "x-request-id",
        request_id.parse().unwrap_or_else(|_| {
            axum::http::HeaderValue::from_static("invalid")
        }),
    );

    let mut response = next.run(request).await;
    response.headers_mut().insert(
        "x-request-id",
        axum::http::HeaderValue::from_str(&request_id)
            .unwrap_or_else(|_| axum::http::HeaderValue::from_static("invalid")),
    );
    response
}
