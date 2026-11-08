//! Health and readiness endpoints.
//!
//! # Spring Boot Comparison
//!
//! In Spring Boot, Spring Boot Actuator exposes `/actuator/health`, `/actuator/health/liveness`,
//! and `/actuator/health/readiness` automatically.
//!
//! In Axum, we implement lightweight handlers that return standard JSON envelopes
//! wrapped in `ApiResponse`. In Kubernetes:
//! - Liveness probe points to `/health` (verifies the process is running)
//! - Readiness probe points to `/health/ready` (verifies dependencies like PostgreSQL are connected)

use crate::shared::response::ApiResponse;
use axum::Json;
use axum::response::IntoResponse;
use serde::Serialize;

/// Response payload for the liveness check.
#[derive(Debug, Serialize)]
pub struct HealthStatus {
    pub status: &'static str,
    pub version: &'static str,
}

/// Response payload for the readiness check.
#[derive(Debug, Serialize)]
pub struct ReadinessStatus {
    pub status: &'static str,
    pub database: &'static str,
}

/// Handler for `GET /health` (Liveness probe).
pub async fn health_check() -> impl IntoResponse {
    let payload = HealthStatus {
        status: "UP",
        version: env!("CARGO_PKG_VERSION"),
    };
    Json(ApiResponse::success(payload))
}

/// Handler for `GET /health/ready` (Readiness probe).
///
/// In Phase 1, returns ready for HTTP service.
/// In Phase 2, this will actively ping the PostgreSQL connection pool.
pub async fn readiness_check() -> impl IntoResponse {
    let payload = ReadinessStatus {
        status: "UP",
        database: "PENDING_PHASE_2",
    };
    Json(ApiResponse::success(payload))
}
