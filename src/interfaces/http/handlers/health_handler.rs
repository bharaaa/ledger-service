//! Health and readiness endpoints.
//!
//! # Spring Boot Comparison
//!
//! In Spring Boot, Spring Boot Actuator exposes `/actuator/health`, `/actuator/health/liveness`,
//! and `/actuator/health/readiness` automatically. The readiness indicator pings DataSource
//! automatically to verify PostgreSQL connectivity.
//!
//! In Axum, we implement:
//! - `GET /health` (Liveness probe): Verifies process is alive and responding to HTTP.
//! - `GET /health/ready` (Readiness probe): Actively pings PostgreSQL with `check_health(&db_pool)`
//!   (`SELECT 1`). Returns HTTP 200 when database is healthy, or HTTP 503 when unreachable.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Serialize;

use crate::infrastructure::database::pool::check_health;
use crate::interfaces::http::state::AppState;
use crate::shared::response::ApiResponse;

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
    (StatusCode::OK, Json(ApiResponse::success(payload)))
}

/// Handler for `GET /health/ready` (Readiness probe).
///
/// Actively queries PostgreSQL with `SELECT 1`.
/// If successful, returns 200 OK with `database: "CONNECTED"`.
/// If database is unreachable, returns 503 Service Unavailable with `database: "DISCONNECTED"`.
pub async fn readiness_check(State(state): State<AppState>) -> impl IntoResponse {
    match check_health(&state.db_pool).await {
        Ok(()) => {
            let payload = ReadinessStatus {
                status: "UP",
                database: "CONNECTED",
            };
            (StatusCode::OK, Json(ApiResponse::success(payload))).into_response()
        }
        Err(err) => {
            tracing::warn!(error = %err, "Readiness check failed: database unreachable");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiResponse::<()>::error(
                    "DATABASE_UNAVAILABLE",
                    "Database connection check failed",
                )),
            )
                .into_response()
        }
    }
}
