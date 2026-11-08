//! HTTP route definitions and router construction.
//!
//! # Spring Boot Comparison
//!
//! In Spring Boot, routes are defined via annotations on controllers:
//! `@GetMapping("/health")`, `@RequestMapping("/api/v1/accounts")`.
//!
//! In Axum, routes are composed into a `Router` tree using `.route()` and `.nest()`.
//! Middleware layers (Tower services) wrap the router via `.layer()`.

use axum::Router;
use axum::middleware as axum_mw;
use axum::routing::get;
use tower_http::trace::TraceLayer;

use crate::interfaces::http::handlers::health_handler::{health_check, readiness_check};
use crate::interfaces::http::middleware::request_id::request_id_middleware;
use crate::interfaces::http::state::AppState;

/// Construct the root application router with all routes and middleware layers.
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health endpoints (unversioned, standard Kubernetes paths)
        .route("/health", get(health_check))
        .route("/health/ready", get(readiness_check))
        // Shared state for all handlers
        .with_state(state)
        // Middleware layers (applied outer-to-inner):
        // 1. Request ID generation and header propagation
        .layer(axum_mw::from_fn(request_id_middleware))
        // 2. Structured request/response tracing
        .layer(TraceLayer::new_for_http())
}
