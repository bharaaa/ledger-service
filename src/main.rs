//! Entry point for the Ledger banking transaction service.
//!
//! # Spring Boot Comparison
//!
//! In Spring Boot, the entry point is an annotated class:
//! ```java
//! @SpringBootApplication
//! public class LedgerApplication {
//!     public static void main(String[] args) {
//!         SpringApplication.run(LedgerApplication.class, args);
//!     }
//! }
//! ```
//!
//! In Rust:
//! - `#[tokio::main]` macro transforms `async fn main()` into a synchronous `fn main()`
//!   that instantiates a multi-threaded Tokio runtime (`tokio::runtime::Runtime`).
//! - Environment variables are loaded via `dotenvy::dotenv()`.
//! - Tracing (logging) is initialized before any application logic executes.
//! - Execution delegates to [`ledger_service::bootstrap::run()`].

use dotenvy::dotenv;
use ledger_service::bootstrap;
use ledger_service::observability::logging::init_logging;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load local .env file if present (silently ignores if not found)
    let _ = dotenv();

    // Initialize structured JSON logging via tracing-subscriber
    init_logging();

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        "Starting Ledger Service"
    );

    // Run the server bootstrap lifecycle
    bootstrap::run().await
}
