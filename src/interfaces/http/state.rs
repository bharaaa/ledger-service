//! Shared application state for Axum request handlers.
//!
//! # Spring Boot Comparison
//!
//! In Spring Boot, components are singletons managed by the `ApplicationContext`
//! and injected into `@RestController` instances via constructor injection (`@Autowired`).
//!
//! In Axum, shared state is encapsulated in an `AppState` struct and passed to the router
//! using `.with_state(app_state)`. Handlers extract this state using the `State` extractor:
//! ```rust,ignore
//! async fn my_handler(State(state): State<AppState>) -> impl IntoResponse { ... }
//! ```
//!
//! Axum clones the state for each request, so any large or non-trivially-cloneable
//! dependencies should be wrapped in `Arc` (e.g., `Arc<dyn AccountService>`).
//! In `AppState`, `HttpClient` and SQLx `PgPool` already wrap `Arc` internally.

use crate::config::settings::Settings;
use crate::infrastructure::external::http_client::HttpClient;

/// Application state shared across all HTTP request handlers.
#[derive(Debug, Clone)]
pub struct AppState {
    pub settings: Settings,
    pub http_client: HttpClient,
}

impl AppState {
    /// Create a new `AppState` instance.
    pub fn new(settings: Settings, http_client: HttpClient) -> Self {
        Self {
            settings,
            http_client,
        }
    }
}
