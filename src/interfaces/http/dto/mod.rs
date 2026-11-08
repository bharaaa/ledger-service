//! HTTP Request and Response Data Transfer Objects (DTOs).
//!
//! # Spring Boot Comparison
//!
//! In Spring Boot, you create POJOs annotated with Jackson (`@JsonProperty`)
//! and Bean Validation annotations (`@NotBlank`, `@Positive`).
//!
//! In Rust, DTOs are structs with `#[derive(Deserialize, Validate)]` for requests
//! and `#[derive(Serialize)]` for responses.
//!
//! DTO submodules will be implemented in Phase 3 & 4:
//! - `account_dto`
//! - `transaction_dto`
