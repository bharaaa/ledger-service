//! Ledger — A modern banking transaction service in Rust.
//!
//! # Architecture
//!
//! This service is structured according to Clean Architecture / Hexagonal Architecture:
//!
//! - [`domain`] — Core business entities, value objects, and repository interfaces
//! - [`application`] — Use cases, commands, queries, and service orchestration
//! - [`infrastructure`] — Database access (SQLx), external HTTP (Reqwest), repository implementations
//! - [`interfaces`] — HTTP API (Axum), DTOs, request middleware, routing
//! - [`config`] — Environment-based configuration
//! - [`observability`] — Structured logging and diagnostics
//! - [`shared`] — Error types, API response envelope, pagination, common aliases

pub mod application;
pub mod bootstrap;
pub mod config;
pub mod domain;
pub mod infrastructure;
pub mod interfaces;
pub mod observability;
pub mod shared;
