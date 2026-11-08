//! Application bootstrapping and server lifecycle management.
//!
//! # Spring Boot Comparison
//!
//! In Spring Boot, `SpringApplication.run(Application.class, args)` discovers
//! beans, sets up the embedded Tomcat/Netty container, executes Flyway/Liquibase
//! migrations, hooks shutdown hooks, and starts listening for HTTP requests.
//!
//! In Rust, `bootstrap::run()` performs these steps explicitly:
//! 1. Loads configuration from environment variables
//! 2. Initializes connection pools and verifies PostgreSQL connectivity
//! 3. Executes embedded SQL migrations (`sqlx::migrate!()`)
//! 4. Initializes outbound HTTP client
//! 5. Constructs the Axum application router with shared `AppState`
//! 6. Binds the Tokio TCP listener
//! 7. Configures graceful shutdown via OS signals (SIGINT / SIGTERM)

use anyhow::{Context, Result};
use tokio::net::TcpListener;

use crate::config::settings::Settings;
use crate::infrastructure::database::pool::{create_pool, run_migrations};
use crate::infrastructure::external::http_client::HttpClient;
use crate::interfaces::http::routes::create_router;
use crate::interfaces::http::state::AppState;

/// Initialize and run the HTTP server until a shutdown signal is received.
pub async fn run() -> Result<()> {
    // 1. Load application configuration
    let settings = Settings::load().context("Failed to load application settings")?;
    let bind_addr = settings.server.addr();

    tracing::info!(
        host = %settings.server.host,
        port = %settings.server.port,
        "Configuration loaded successfully"
    );

    // 2. Initialize infrastructure dependencies: PostgreSQL connection pool
    let db_pool = create_pool(&settings.database)
        .await
        .context("Failed to connect to PostgreSQL database")?;

    tracing::info!("Connected to PostgreSQL connection pool successfully");

    // 3. Run embedded database migrations automatically at startup
    run_migrations(&db_pool)
        .await
        .context("Failed to run database migrations")?;

    tracing::info!("Database migrations applied successfully");

    // 4. Initialize outbound HTTP client
    let http_client = HttpClient::new().context("Failed to initialize outbound HTTP client")?;

    // 5. Assemble shared application state
    let app_state = AppState::new(settings, http_client, db_pool);

    // 6. Build Axum router
    let app = create_router(app_state);

    // 7. Bind TCP listener with Tokio
    let listener = TcpListener::bind(&bind_addr)
        .await
        .with_context(|| format!("Failed to bind TCP listener to {bind_addr}"))?;

    tracing::info!("Server listening on http://{}", bind_addr);

    // 8. Start server with graceful shutdown handling
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("HTTP server encountered an unrecoverable error")?;

    tracing::info!("Server shutdown completed cleanly");
    Ok(())
}

/// Listen for OS signals to initiate graceful shutdown.
///
/// Handles both:
/// - `SIGINT` (Ctrl+C from terminal)
/// - `SIGTERM` (sent by Docker / Kubernetes on container stop)
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C signal handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received SIGINT (Ctrl+C)");
        },
        _ = terminate => {
            tracing::info!("Received SIGTERM");
        },
    }

    tracing::info!("Initiating graceful shutdown, draining active connections...");
}
