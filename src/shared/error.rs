//! Application error types and HTTP error conversion.
//!
//! # Error Handling Strategy
//!
//! Ledger uses a three-layer error model:
//!
//! 1. **Domain errors** — business rule violations (e.g., insufficient balance)
//! 2. **Infrastructure errors** — database failures, network issues
//! 3. **HTTP errors** — converted from the above via `IntoResponse`
//!
//! # Spring Boot Comparison
//!
//! In Spring Boot, you'd define custom exception classes and handle them with
//! `@ControllerAdvice` + `@ExceptionHandler`. The framework catches exceptions
//! thrown from anywhere in the call stack and converts them to HTTP responses.
//!
//! In Rust, there are no exceptions. Functions return `Result<T, E>`, and errors
//! propagate explicitly via the `?` operator. The `IntoResponse` implementation
//! below is the Axum equivalent of `@ExceptionHandler` — it converts our error
//! type into an HTTP response with the correct status code and JSON body.
//!
//! # Key Rust Concept — `thiserror` Derive Macro
//!
//! `#[derive(thiserror::Error)]` auto-implements `std::error::Error` and `Display`
//! for our enum. Each `#[error("...")]` attribute defines the display message.
//! The `#[from]` attribute generates `From` impls for automatic conversion.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::shared::response::ApiResponse;

/// Application-wide error type.
///
/// All errors in the system are funneled through this enum. Each variant maps
/// to a specific HTTP status code and error code for the API response.
///
/// # Design Decision
///
/// We use a single error enum rather than separate error types per layer because:
/// 1. Axum handlers need a single error type for `Result<T, AppError>`
/// 2. The `?` operator can convert infrastructure errors (SQLx) into `AppError` automatically
/// 3. It keeps the error handling centralized and auditable
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    // -----------------------------------------------------------------------
    // Account errors
    // -----------------------------------------------------------------------
    #[error("Account not found")]
    AccountNotFound,

    #[error("Account is not active")]
    AccountNotActive,

    #[error("Insufficient account balance")]
    InsufficientBalance,

    #[error("Cannot transfer to the same account")]
    SameAccountTransfer,

    #[error("Currency mismatch between accounts")]
    CurrencyMismatch,

    // -----------------------------------------------------------------------
    // Transaction errors
    // -----------------------------------------------------------------------
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),

    #[error("Transaction not found")]
    TransactionNotFound,

    // -----------------------------------------------------------------------
    // Idempotency errors
    // -----------------------------------------------------------------------
    #[error("Duplicate idempotency key with different payload")]
    IdempotencyConflict,

    #[error("Missing Idempotency-Key header")]
    MissingIdempotencyKey,

    // -----------------------------------------------------------------------
    // Validation errors
    // -----------------------------------------------------------------------
    #[error("Validation error: {0}")]
    Validation(String),

    // -----------------------------------------------------------------------
    // Infrastructure errors
    // -----------------------------------------------------------------------
    /// Database errors. The `#[from]` attribute auto-generates:
    /// `impl From<sqlx::Error> for AppError`
    ///
    /// This means any `sqlx::Error` can be converted to `AppError::Database`
    /// automatically when using the `?` operator.
    #[error("Database error")]
    Database(#[from] sqlx::Error),

    /// Catch-all for unexpected internal errors.
    /// `anyhow::Error` can wrap any error type with additional context.
    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),
}

impl AppError {
    /// Map each error variant to an HTTP status code.
    ///
    /// This is the Axum equivalent of mapping exception types to
    /// `@ResponseStatus` annotations in Spring Boot.
    pub fn status_code(&self) -> StatusCode {
        match self {
            // 404 Not Found
            Self::AccountNotFound | Self::TransactionNotFound => StatusCode::NOT_FOUND,

            // 422 Unprocessable Entity — the request was well-formed but violates business rules
            Self::AccountNotActive
            | Self::InsufficientBalance
            | Self::SameAccountTransfer
            | Self::CurrencyMismatch
            | Self::InvalidAmount(_) => StatusCode::UNPROCESSABLE_ENTITY,

            // 409 Conflict — idempotency key reused with different payload
            Self::IdempotencyConflict => StatusCode::CONFLICT,

            // 400 Bad Request — malformed or invalid input
            Self::MissingIdempotencyKey | Self::Validation(_) => StatusCode::BAD_REQUEST,

            // 500 Internal Server Error — never expose details to clients
            Self::Database(_) | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Machine-readable error code for the JSON response.
    ///
    /// Clients can switch on this code for programmatic error handling,
    /// rather than parsing human-readable messages.
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::AccountNotFound => "ACCOUNT_NOT_FOUND",
            Self::AccountNotActive => "ACCOUNT_NOT_ACTIVE",
            Self::InsufficientBalance => "INSUFFICIENT_BALANCE",
            Self::SameAccountTransfer => "SAME_ACCOUNT_TRANSFER",
            Self::CurrencyMismatch => "CURRENCY_MISMATCH",
            Self::InvalidAmount(_) => "INVALID_AMOUNT",
            Self::TransactionNotFound => "TRANSACTION_NOT_FOUND",
            Self::IdempotencyConflict => "IDEMPOTENCY_CONFLICT",
            Self::MissingIdempotencyKey => "MISSING_IDEMPOTENCY_KEY",
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::Database(_) => "DATABASE_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }
}

/// Convert `AppError` into an Axum HTTP response.
///
/// # Rust Concept — `IntoResponse` Trait
///
/// Axum calls this automatically when a handler returns `Err(AppError)`.
/// This is the compile-time equivalent of Spring Boot's `@ExceptionHandler`:
/// - In Spring: runtime reflection finds the right handler method
/// - In Rust: the compiler generates the conversion code at compile time
///
/// # Security
///
/// Infrastructure errors (Database, Internal) are logged with full details
/// but return a generic message to the client. This prevents leaking SQL
/// queries, stack traces, or internal architecture details.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Log the error with appropriate severity
        match &self {
            Self::Database(e) => {
                tracing::error!(error = %e, "Database error occurred");
            }
            Self::Internal(e) => {
                tracing::error!(error = %e, "Internal error occurred");
            }
            _ => {
                tracing::warn!(
                    error = %self,
                    code = self.error_code(),
                    "Request error"
                );
            }
        }

        let status = self.status_code();

        // Sanitize error messages — never expose internal details
        let message = match &self {
            Self::Database(_) | Self::Internal(_) => {
                "An internal error occurred. Please try again later.".to_string()
            }
            other => other.to_string(),
        };

        let body = ApiResponse::<()>::error(self.error_code(), &message);
        (status, axum::Json(body)).into_response()
    }
}

/// Type alias for handler results.
///
/// Handlers return `AppResult<T>` instead of `Result<T, AppError>` for brevity.
/// This is a common Rust pattern for reducing boilerplate.
pub type AppResult<T> = Result<T, AppError>;
