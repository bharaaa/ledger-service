//! Consistent API response envelope.
//!
//! Every API response uses this wrapper to provide a uniform contract for clients.
//!
//! # Spring Boot Comparison
//!
//! In Spring Boot, you might create a generic `ApiResponse<T>` wrapper and return
//! `ResponseEntity<ApiResponse<T>>` from every controller. Some teams use
//! `@ControllerAdvice` to wrap all responses automatically.
//!
//! In Rust/Axum, we return `Json<ApiResponse<T>>` from handlers. The `IntoResponse`
//! trait on `AppError` ensures error responses also use this format.
//!
//! # JSON Format
//!
//! Success:
//! ```json
//! {
//!   "success": true,
//!   "data": { ... },
//!   "error": null,
//!   "meta": null
//! }
//! ```
//!
//! Error:
//! ```json
//! {
//!   "success": false,
//!   "data": null,
//!   "error": { "code": "INSUFFICIENT_BALANCE", "message": "..." },
//!   "meta": null
//! }
//! ```

use serde::Serialize;

/// Uniform API response envelope.
///
/// Generic over `T` so it works with any serializable data type.
/// The `#[serde(skip_serializing_if = "Option::is_none")]` attribute
/// omits null fields from the JSON output for cleaner responses.
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorDetail>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

/// Error detail included in error responses.
#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    /// Machine-readable error code (e.g., "INSUFFICIENT_BALANCE").
    pub code: String,
    /// Human-readable error message.
    pub message: String,
}

impl<T: Serialize> ApiResponse<T> {
    /// Create a success response wrapping the given data.
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            meta: None,
        }
    }

    /// Create a success response with pagination or other metadata.
    pub fn success_with_meta(data: T, meta: serde_json::Value) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            meta: Some(meta),
        }
    }

    /// Create an error response with a machine-readable code and human-readable message.
    pub fn error(code: &str, message: &str) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(ErrorDetail {
                code: code.to_string(),
                message: message.to_string(),
            }),
            meta: None,
        }
    }
}
