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
//!
//! # Database Migrations
//!
//! While Spring Boot uses Flyway or Liquibase via runtime classpath scanning, SQLx embeds
//! migration scripts directly into the binary at compile time via `sqlx::migrate!("./migrations")`.
//! This ensures zero external file dependencies when deploying the application container.

use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

use crate::config::settings::DatabaseSettings;

/// Type alias for our PostgreSQL connection pool.
pub type DbPool = PgPool;

/// Create a new PostgreSQL connection pool and eagerly verify connection.
pub async fn create_pool(settings: &DatabaseSettings) -> Result<DbPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(settings.max_connections)
        .min_connections(settings.min_connections)
        .acquire_timeout(Duration::from_secs(settings.acquire_timeout_seconds))
        .connect(&settings.url)
        .await
}

/// Create a new PostgreSQL connection pool lazily without blocking on startup.
///
/// Connections will be established asynchronously on first request.
/// Useful for fast startup and unit testing.
pub fn create_pool_lazy(settings: &DatabaseSettings) -> Result<DbPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(settings.max_connections)
        .min_connections(settings.min_connections)
        .acquire_timeout(Duration::from_secs(settings.acquire_timeout_seconds))
        .connect_lazy(&settings.url)
}

/// Execute all embedded SQL migrations against the database.
///
/// This applies `001_create_users.sql` through `005_create_idempotency_keys.sql`
/// sequentially inside a transaction, recording completed migrations in the
/// `_sqlx_migrations` table.
pub async fn run_migrations(pool: &DbPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}

/// Check database connectivity and responsiveness (ping).
///
/// Executes a minimal `SELECT 1` query.
/// Used by readiness health check probes (`GET /health/ready`).
pub async fn check_health(pool: &DbPool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1").execute(pool).await?;
    Ok(())
}
