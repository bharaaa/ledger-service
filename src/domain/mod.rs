//! Domain layer — the heart of the business logic.
//!
//! This layer contains entities, value objects, and repository trait definitions.
//! It has **no dependencies** on infrastructure (database, HTTP, external services).
//!
//! # Clean Architecture Rule
//!
//! The domain layer is the innermost circle. It defines the "what" (business rules)
//! without knowing the "how" (database queries, HTTP calls). Repository traits
//! are defined here but implemented in the infrastructure layer.
//!
//! Modules will be added in Phase 3:
//! - `account` — Account entity, value objects, repository trait
//! - `transaction` — Transaction entity, value objects, repository trait
//! - `ledger` — LedgerEntry entity, repository trait
