//! Request ID generation and propagation middleware.
//!
//! # Spring Boot Comparison
//!
//! In Spring Boot, request ID propagation is typically done with a `Filter` or `HandlerInterceptor`
//! that inspects or generates a correlation ID and places it into SLF4J's `MDC`.
//!
//! In Axum, this is implemented as an async middleware function (`axum::middleware::from_fn`)
//! that intercepts requests before reaching handlers, ensures an `x-request-id` header
//! is attached to both request and response, and records the ID in tracing events.

use axum::{
    extract::Request,
    http::{HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

/// Header name used for distributed tracing and request correlation.
pub static REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");

/// Middleware that extracts or generates a unique UUIDv7 request ID.
///
/// If an incoming request already contains `x-request-id`, it is preserved.
/// Otherwise, a new time-ordered UUIDv7 is generated.
/// The ID is inserted into request headers (for downstream handlers/tracing)
/// and attached to the outgoing response headers.
pub async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    let request_id = request
        .headers()
        .get(&REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::now_v7().to_string());

    let header_value = HeaderValue::from_str(&request_id)
        .unwrap_or_else(|_| HeaderValue::from_static("unknown-id"));

    request
        .headers_mut()
        .insert(REQUEST_ID_HEADER.clone(), header_value.clone());

    let mut response = next.run(request).await;

    response
        .headers_mut()
        .insert(REQUEST_ID_HEADER.clone(), header_value);

    response
}
