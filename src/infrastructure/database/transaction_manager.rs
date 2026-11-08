//! Transaction manager abstraction.
//!
//! # Spring Boot Comparison
//!
//! In Spring Boot, `@Transactional` uses Spring's AOP proxy mechanism and
//! `PlatformTransactionManager` to bind database transactions to the current thread
//! (via ThreadLocal).
//!
//! In Rust, there are no thread-local transaction contexts. Transactions are explicit
//! values (`sqlx::Transaction<'_, Postgres>`) passed or managed in async scopes.
//!
//! This module will provide transactional execution helpers in Phase 4.
