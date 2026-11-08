//! Shared type aliases and common types.
//!
//! This module will be populated with domain-wide types (Money, Currency, etc.)
//! in Phase 3 and Phase 4. For now, it provides foundational type aliases.

/// Type alias for database IDs throughout the application.
///
/// Using UUIDv7 (time-ordered) for primary keys because:
/// 1. Globally unique without coordination (unlike auto-increment)
/// 2. Chronologically sortable (unlike UUIDv4)
/// 3. Efficient B-tree index locality (unlike random UUIDv4)
///
/// In Java, you'd use `java.util.UUID`. Rust's `uuid` crate with
/// the `v7` feature provides time-ordered UUID generation.
pub type Id = uuid::Uuid;
