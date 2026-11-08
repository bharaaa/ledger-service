//! Structured logging initialization.
//!
//! # Tracing vs Traditional Logging
//!
//! Rust's `tracing` crate is more than a logger — it's a structured diagnostics framework.
//! Key differences from Java's SLF4J + Logback:
//!
//! | Concept | Java (SLF4J/Logback) | Rust (tracing) |
//! |---|---|---|
//! | Log statement | `log.info("message")` | `tracing::info!("message")` |
//! | Structured fields | MDC (`MDC.put("key", "val")`) | Span fields (`info!(key = "val")`) |
//! | Context propagation | ThreadLocal MDC | `Span` hierarchy (async-aware) |
//! | Output format | Logback XML config | Subscriber layers (code-based) |
//!
//! The key advantage of `tracing` is that it's **async-aware**: spans propagate
//! correctly across `.await` points, even when tasks move between threads.
//! Java's MDC (ThreadLocal) breaks in async/reactive code — Spring WebFlux needs
//! special `ContextView` propagation. Rust solves this at the framework level.
//!
//! # OpenTelemetry Readiness
//!
//! The `tracing` subscriber architecture supports layered composition. Adding
//! OpenTelemetry export is as simple as adding a `tracing-opentelemetry` layer
//! to the subscriber stack — no changes to application instrumentation needed.

use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize the global tracing subscriber with structured JSON output.
///
/// This should be called once at application startup, before any other
/// tracing instrumentation. It configures:
///
/// 1. **Environment filter**: Controls log levels via `RUST_LOG` env var.
///    Examples:
///    - `RUST_LOG=info` — all crates at info level
///    - `RUST_LOG=ledger_service=debug,tower_http=info` — per-crate levels
///    - `RUST_LOG=ledger_service::infrastructure=trace` — per-module levels
///
/// 2. **JSON formatter**: Outputs structured JSON logs suitable for
///    log aggregation systems (ELK, Datadog, CloudWatch, etc.).
///
/// # Panics
///
/// Panics if a global subscriber has already been set (i.e., called twice).
pub fn init_logging() {
    // Build the environment filter from RUST_LOG or fall back to "info"
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // JSON formatting layer for structured logs
    let fmt_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_target(true) // Include module path (e.g., "ledger_service::shared::error")
        .with_thread_ids(true) // Useful for debugging async task scheduling
        .with_file(true) // Source file for each log event
        .with_line_number(true) // Line number for each log event
        .with_span_list(true); // Include parent span context

    // Compose the subscriber from layers
    //
    // Rust concept — Layer composition:
    // Subscribers are built by stacking layers. Each layer can filter, transform,
    // or export trace events independently. This is similar to Logback's appender
    // pipeline, but composed in code rather than XML.
    //
    // Future: Add OpenTelemetry layer here:
    //   .with(tracing_opentelemetry::layer().with_exporter(exporter))
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
}
