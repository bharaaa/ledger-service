//! Application layer — use case orchestration.
//!
//! This layer contains services that coordinate domain objects to fulfill
//! business use cases (commands and queries).
//!
//! # Spring Boot Comparison
//!
//! Application services are equivalent to Spring `@Service` classes.
//! They receive repository interfaces (traits) via constructor injection
//! and orchestrate domain operations.
//!
//! Key difference: In Spring Boot, services are managed as singleton beans
//! by the IoC container. In Rust, services are plain structs held in `AppState`
//! and shared via `Arc` (which Axum's `State` extractor handles internally).
//!
//! Modules will be added in Phase 3:
//! - `account` — Account service, commands, queries
//! - `transaction` — Transaction service, commands, queries
