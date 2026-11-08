//! Application configuration loaded from environment variables.
//!
//! # Spring Boot Comparison
//!
//! In Spring Boot, configuration is loaded from `application.yml` or `application.properties`
//! and bound to `@ConfigurationProperties` classes. Spring's IoC container manages the
//! lifecycle and injection.
//!
//! In Rust, we load configuration explicitly at startup. There's no runtime reflection or
//! classpath scanning — everything is explicit and type-checked at compile time.
//!
//! # Environment Variables
//!
//! | Variable | Default | Description |
//! |---|---|---|
//! | `SERVER_HOST` | `0.0.0.0` | Server bind address |
//! | `SERVER_PORT` | `8080` | Server port |
//! | `DATABASE_URL` | *(see code)* | PostgreSQL connection string |
//! | `DATABASE_MAX_CONNECTIONS` | `10` | Max pool connections |
//! | `DATABASE_MIN_CONNECTIONS` | `2` | Min pool connections |
//! | `DATABASE_ACQUIRE_TIMEOUT_SECONDS` | `5` | Connection acquire timeout |

use anyhow::{Context, Result};

/// Top-level application settings.
///
/// This struct is constructed once at startup and shared (via `AppState`) with all
/// request handlers. It is intentionally immutable after construction.
///
/// In Spring Boot, this would be a `@ConfigurationProperties` class.
#[derive(Debug, Clone)]
pub struct Settings {
    pub server: ServerSettings,
    pub database: DatabaseSettings,
}

/// HTTP server configuration.
#[derive(Debug, Clone)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
}

/// Database connection pool configuration.
///
/// These values map directly to SQLx's `PgPoolOptions`:
/// - `max_connections` → `PgPoolOptions::max_connections()`
/// - `min_connections` → `PgPoolOptions::min_connections()`
/// - `acquire_timeout_seconds` → `PgPoolOptions::acquire_timeout()`
#[derive(Debug, Clone)]
pub struct DatabaseSettings {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout_seconds: u64,
}

impl Settings {
    /// Load settings from environment variables with sensible defaults.
    ///
    /// Uses `dotenvy` to load `.env` files in development.
    /// In production (Docker/Kubernetes), environment variables are set directly.
    ///
    /// # Errors
    ///
    /// Returns an error if any environment variable has an invalid format
    /// (e.g., `SERVER_PORT=abc`).
    pub fn load() -> Result<Self> {
        Ok(Self {
            server: ServerSettings {
                host: env_or("SERVER_HOST", "0.0.0.0"),
                port: parse_env("SERVER_PORT", "8080")?,
            },
            database: DatabaseSettings {
                url: env_or(
                    "DATABASE_URL",
                    "postgres://postgres:postgres@localhost:5432/ledger",
                ),
                max_connections: parse_env("DATABASE_MAX_CONNECTIONS", "10")?,
                min_connections: parse_env("DATABASE_MIN_CONNECTIONS", "2")?,
                acquire_timeout_seconds: parse_env("DATABASE_ACQUIRE_TIMEOUT_SECONDS", "5")?,
            },
        })
    }
}

impl ServerSettings {
    /// Returns the socket address string for binding the TCP listener.
    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

/// Read an environment variable with a fallback default.
fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Parse an environment variable into a typed value, with a fallback default.
///
/// # Rust Concept — Trait Bounds
///
/// The `T: std::str::FromStr` bound means this function works with any type that
/// can be parsed from a string (u16, u32, u64, bool, etc.). This is similar to
/// Java generics with `<T extends Parseable>`, but resolved entirely at compile time
/// through monomorphization — the compiler generates a specialized version for each
/// concrete type used.
fn parse_env<T>(key: &str, default: &str) -> Result<T>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    env_or(key, default)
        .parse()
        .with_context(|| format!("{key} must be a valid {}", std::any::type_name::<T>()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_or_returns_default() {
        // Use a key that definitely doesn't exist
        let result = env_or("LEDGER_TEST_NONEXISTENT_VAR_12345", "fallback");
        assert_eq!(result, "fallback");
    }

    #[test]
    fn test_parse_env_with_default() {
        let result: u16 = parse_env("LEDGER_TEST_NONEXISTENT_PORT_12345", "3000").unwrap();
        assert_eq!(result, 3000);
    }

    #[test]
    fn test_parse_env_invalid_value() {
        // SAFETY: In Rust 2024, set_var and remove_var are unsafe due to potential
        // data races in multi-threaded environments. This is a single-threaded test.
        unsafe {
            std::env::set_var("LEDGER_TEST_BAD_PORT", "not_a_number");
        }
        let result: Result<u16> = parse_env("LEDGER_TEST_BAD_PORT", "8080");
        assert!(result.is_err());
        unsafe {
            std::env::remove_var("LEDGER_TEST_BAD_PORT");
        }
    }

    #[test]
    fn test_server_addr() {
        let server = ServerSettings {
            host: "127.0.0.1".to_string(),
            port: 3000,
        };
        assert_eq!(server.addr(), "127.0.0.1:3000");
    }
}
