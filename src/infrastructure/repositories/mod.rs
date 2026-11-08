//! Infrastructure repository implementations.
//!
//! # Spring Boot Comparison
//!
//! In Spring Boot with Spring Data, you define interfaces extending `CrudRepository`
//! or `JpaRepository`, and Spring synthesizes proxy implementations at runtime.
//!
//! In Rust with Clean Architecture, domain traits (defined in `src/domain/*/repository.rs`)
//! are explicitly implemented here using SQLx queries.
//!
//! Modules will be added in Phase 3 & 4:
//! - `account_repository`
//! - `transaction_repository`
//! - `ledger_repository`
