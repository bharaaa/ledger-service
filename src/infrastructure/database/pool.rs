//! Database connection pool configuration and creation.
//!
//! # Spring Boot Comparison
//!
//! In Spring Boot, HikariCP is the default connection pool managed automatically by
//! `spring-boot-starter-data-jdbc` or `spring-boot-starter-data-jpa`. You configure pool
//! properties in `application.yml` (`spring.datasource.hikari.*`).
//!
//! In Rust with SQLx, we explicitly build a `sqlx::PgPool` using `sqlx::postgres::PgPoolOptions`.
//! The pool is asynchronous, thread-safe, and cheaply cloneable (it internally uses `Arc`).

use crate::config::settings::DatabaseSettings;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

/// Type alias for our PostgreSQL connection pool.
pub type DbPool = PgPool;

/// Create a new PostgreSQL connection pool from settings.
pub async fn create_pool(settings: &DatabaseSettings) -> Result<DbPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(settings.max_connections)
        .min_connections(settings.min_connections)
        .acquire_timeout(Duration::from_secs(settings.acquire_timeout_seconds))
        .connect(&settings.url)
        .await
}
