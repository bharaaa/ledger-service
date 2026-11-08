//! API Integration Tests for Health and Middleware.
//!
//! # Spring Boot Comparison
//!
//! In Spring Boot, you'd use `@SpringBootTest(webEnvironment = RANDOM_PORT)`
//! or `MockMvc` to test HTTP endpoints and filters.
//!
//! In Axum, we test handlers and middleware using `tower::ServiceExt::oneshot`.
//! This sends simulated HTTP requests directly through the Axum router in-memory
//! without spinning up an actual TCP socket or network listener.
//! Tests run blazingly fast and in parallel.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

use ledger_service::config::settings::{DatabaseSettings, ServerSettings, Settings};
use ledger_service::infrastructure::external::http_client::HttpClient;
use ledger_service::interfaces::http::middleware::request_id::REQUEST_ID_HEADER;
use ledger_service::interfaces::http::routes::create_router;
use ledger_service::interfaces::http::state::AppState;

fn test_app_state() -> AppState {
    let settings = Settings {
        server: ServerSettings {
            host: "127.0.0.1".to_string(),
            port: 8080,
        },
        database: DatabaseSettings {
            url: "postgres://postgres:postgres@localhost:5432/ledger_test".to_string(),
            max_connections: 5,
            min_connections: 1,
            acquire_timeout_seconds: 3,
        },
    };
    let http_client = HttpClient::new().expect("Failed to build test HttpClient");
    AppState::new(settings, http_client)
}

#[tokio::test]
async fn test_health_check_endpoint() {
    let app = create_router(test_app_state());

    let request = Request::builder()
        .uri("/health")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify x-request-id header is automatically generated and attached
    assert!(response.headers().contains_key(&REQUEST_ID_HEADER));

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["data"]["status"], "UP");
    assert_eq!(json["data"]["version"], env!("CARGO_PKG_VERSION"));
    assert!(json["error"].is_null());
}

#[tokio::test]
async fn test_readiness_check_endpoint() {
    let app = create_router(test_app_state());

    let request = Request::builder()
        .uri("/health/ready")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().contains_key(&REQUEST_ID_HEADER));

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["data"]["status"], "UP");
}

#[tokio::test]
async fn test_request_id_propagation() {
    let app = create_router(test_app_state());
    let custom_id = "test-correlation-id-12345";

    let request = Request::builder()
        .uri("/health")
        .method("GET")
        .header(&REQUEST_ID_HEADER, custom_id)
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Verify incoming request ID was preserved in response
    let header_val = response.headers().get(&REQUEST_ID_HEADER).unwrap();
    assert_eq!(header_val.to_str().unwrap(), custom_id);
}

#[tokio::test]
async fn test_not_found_returns_404() {
    let app = create_router(test_app_state());

    let request = Request::builder()
        .uri("/unknown-endpoint")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
