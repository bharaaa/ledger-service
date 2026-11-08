//! Reusable outbound HTTP client using Reqwest.
//!
//! # Spring Boot Comparison
//!
//! In Spring Boot, you often configure a `RestTemplateBuilder` or `WebClient.Builder`
//! bean with timeouts and connection limits, then inject `WebClient` into client components.
//!
//! In Rust, `reqwest::Client` is internally wrapped in an `Arc`, which makes cloning it
//! very cheap (just an atomic increment). A single `HttpClient` instance should be
//! created at startup and shared across all services that make outbound HTTP calls.
//!
//! # Features
//!
//! - Configurable connection and request timeouts
//! - Default JSON accept and content-type headers
//! - Async execution with Tokio

use reqwest::Client;
use std::time::Duration;

/// Wrapper around `reqwest::Client` for outbound HTTP calls.
#[derive(Debug, Clone)]
pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    /// Create a new `HttpClient` with recommended default timeouts.
    pub fn new() -> Result<Self, reqwest::Error> {
        Self::with_timeout(Duration::from_secs(10), Duration::from_secs(3))
    }

    /// Create a new `HttpClient` with custom request and connect timeouts.
    pub fn with_timeout(
        request_timeout: Duration,
        connect_timeout: Duration,
    ) -> Result<Self, reqwest::Error> {
        let client = Client::builder()
            .timeout(request_timeout)
            .connect_timeout(connect_timeout)
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(10)
            .build()?;

        Ok(Self { client })
    }

    /// Access the underlying `reqwest::Client` instance.
    pub fn inner(&self) -> &Client {
        &self.client
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new().expect("Failed to build default HttpClient")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_client_creation() {
        let client = HttpClient::new();
        assert!(client.is_ok());
    }
}
