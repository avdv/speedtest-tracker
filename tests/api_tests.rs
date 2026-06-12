use axum_test::TestServer;
use axum_test::http::{HeaderName, HeaderValue};
use serde_json::Value;
use sha2::Digest;
use speedtest_tracker::{AppState, Database, create_app};
use std::env;

/// Helper function to create a test database and app state
async fn setup_test_app() -> TestServer {
    // Set up test database URL (in-memory SQLite for tests)
    env::set_var("DATABASE_URL", "sqlite::memory:");
    env::set_var("SESSION_SECRET", "test-secret-key-32-characters!!");

    // Create database connection
    let db = Database::connect()
        .await
        .expect("Failed to connect to test database");

    // Run migrations
    if let Database::Sqlite(pool) = &db {
        sqlx::migrate!("./migrations")
            .run(pool)
            .await
            .expect("Failed to run migrations");
    }

    let state = AppState { db };
    let app = create_app(state).await;

    TestServer::new(app).expect("Failed to create test server")
}

/// Helper to setup test state with database (for tests that need to pre-populate data)
async fn setup_test_state() -> AppState {
    env::set_var("DATABASE_URL", "sqlite::memory:");

    let db = Database::connect()
        .await
        .expect("Failed to connect to test database");

    if let Database::Sqlite(pool) = &db {
        sqlx::migrate!("./migrations")
            .run(pool)
            .await
            .expect("Failed to run migrations");
    }

    AppState { db }
}

/// Helper to create a test result in the database
async fn create_test_result(state: &AppState) -> i64 {
    let query = r#"
        INSERT INTO results (service, ping, download, upload, status, scheduled, created_at, updated_at)
        VALUES ('ookla', 12.5, 123456789, 12345678, 'completed', 1, datetime('now'), datetime('now'))
    "#;

    match &state.db {
        #[cfg(feature = "sqlite")]
        Database::Sqlite(pool) => {
            sqlx::query(query)
                .execute(pool)
                .await
                .expect("Failed to insert test result");

            let row: (i64,) = sqlx::query_as("SELECT last_insert_rowid()")
                .fetch_one(pool)
                .await
                .expect("Failed to get last insert id");

            row.0
        }
        #[cfg(any(feature = "postgres", feature = "mysql"))]
        _ => panic!("Only SQLite is supported for tests"),
    }
}

/// Helper to create a test API token
async fn create_test_token(state: &AppState, abilities: &str) -> String {
    let token = format!("test-token-{}", uuid::Uuid::new_v4());
    let token_hash = format!("{:x}", sha2::Sha256::digest(token.as_bytes()));

    let query = r#"
        INSERT INTO personal_access_tokens (name, token, abilities, created_at, updated_at)
        VALUES ('Test Token', ?, ?, datetime('now'), datetime('now'))
    "#;

    match &state.db {
        #[cfg(feature = "sqlite")]
        Database::Sqlite(pool) => {
            sqlx::query(query)
                .bind(&token_hash)
                .bind(abilities)
                .execute(pool)
                .await
                .expect("Failed to insert test token");
        }
        #[cfg(any(feature = "postgres", feature = "mysql"))]
        _ => panic!("Only SQLite is supported for tests"),
    }

    token
}

#[cfg(test)]
mod api_endpoint_tests {
    use super::*;

    // ============================================================================
    // Public Endpoints (No Auth Required)
    // ============================================================================

    #[tokio::test]
    async fn test_healthcheck_returns_ok() {
        let server = setup_test_app().await;

        let response = server.get("/api/healthcheck").await;

        response.assert_status_ok();

        let json: Value = response.json();
        assert_eq!(json["message"], "Speedtest Tracker is running!");
    }

    #[tokio::test]
    async fn test_legacy_latest_endpoint_returns_404_when_no_results() {
        let server = setup_test_app().await;

        let response = server.get("/api/speedtest/latest").await;

        response.assert_status_not_found();
    }

    #[tokio::test]
    async fn test_legacy_latest_endpoint_returns_result() {
        // Create a test result
        let state = setup_test_state().await;
        create_test_result(&state).await;

        // Recreate server with populated database
        let app = create_app(state).await;
        let server = TestServer::new(app).expect("Failed to create test server");

        let response = server.get("/api/speedtest/latest").await;

        response.assert_status_ok();

        let json: Value = response.json();
        assert_eq!(json["message"], "ok");
        assert!(json["data"].is_object());
        assert!(json["data"]["id"].is_number());
        assert_eq!(json["data"]["ping"], 12.5);
        assert_eq!(json["data"]["download"], 100.5); // Converted to Mbps
        assert_eq!(json["data"]["upload"], 50.2); // Converted to Mbps
    }

    // ============================================================================
    // Protected Endpoints (Auth Required)
    // ============================================================================

    #[tokio::test]
    async fn test_v1_results_requires_authentication() {
        let server = setup_test_app().await;

        let response = server.get("/api/v1/results").await;

        // TypedHeader extraction returns 400 (Bad Request) when header is missing
        response.assert_status(axum::http::StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_v1_results_with_valid_token() {
        // Create state with token
        let state = setup_test_state().await;

        let token = create_test_token(&state, "results:read").await;
        create_test_result(&state).await;

        // Recreate server
        let app = create_app(state).await;
        let server = TestServer::new(app).expect("Failed to create test server");

        let response = server
            .get("/api/v1/results")
            .add_header(
                HeaderName::from_static("authorization"),
                HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
            )
            .await;

        response.assert_status_ok();

        let json: Value = response.json();
        assert!(json["data"].is_array());
        assert_eq!(json["page"], 1);
        assert_eq!(json["per_page"], 25);
    }

    #[tokio::test]
    async fn test_v1_results_latest_requires_auth() {
        let server = setup_test_app().await;

        let response = server.get("/api/v1/results/latest").await;

        // TypedHeader extraction returns 400 (Bad Request) when header is missing
        response.assert_status(axum::http::StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_v1_results_latest_with_token() {
        // Setup
        let state = setup_test_state().await;

        let token = create_test_token(&state, "results:read").await;
        create_test_result(&state).await;

        let app = create_app(state).await;
        let server = TestServer::new(app).expect("Failed to create test server");

        let response = server
            .get("/api/v1/results/latest")
            .add_header(
                HeaderName::from_static("authorization"),
                HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
            )
            .await;

        response.assert_status_ok();

        let json: Value = response.json();
        assert!(json["id"].is_number());
        assert_eq!(json["service"], "ookla");
        assert_eq!(json["status"], "completed");
    }

    #[tokio::test]
    async fn test_v1_results_by_id_requires_auth() {
        let server = setup_test_app().await;

        let response = server.get("/api/v1/results/1").await;

        // TypedHeader extraction returns 400 (Bad Request) when header is missing
        response.assert_status(axum::http::StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_v1_results_by_id_returns_404_for_nonexistent() {
        let state = setup_test_state().await;

        let token = create_test_token(&state, "results:read").await;

        let app = create_app(state).await;
        let server = TestServer::new(app).expect("Failed to create test server");

        let response = server
            .get("/api/v1/results/99999")
            .add_header(
                HeaderName::from_static("authorization"),
                HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
            )
            .await;

        response.assert_status_not_found();
    }

    #[tokio::test]
    async fn test_v1_results_by_id_with_valid_id() {
        let state = setup_test_state().await;

        let token = create_test_token(&state, "results:read").await;
        let result_id = create_test_result(&state).await;

        let app = create_app(state).await;
        let server = TestServer::new(app).expect("Failed to create test server");

        let response = server
            .get(&format!("/api/v1/results/{}", result_id))
            .add_header(
                HeaderName::from_static("authorization"),
                HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
            )
            .await;

        response.assert_status_ok();

        let json: Value = response.json();
        assert_eq!(json["id"], result_id);
        assert_eq!(json["service"], "ookla");
    }

    #[tokio::test]
    async fn test_v1_stats_requires_auth() {
        let server = setup_test_app().await;

        let response = server.get("/api/v1/stats").await;

        // TypedHeader extraction returns 400 (Bad Request) when header is missing
        response.assert_status(axum::http::StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_v1_stats_returns_aggregated_data() {
        let state = setup_test_state().await;

        let token = create_test_token(&state, "results:read").await;
        create_test_result(&state).await;

        let app = create_app(state).await;
        let server = TestServer::new(app).expect("Failed to create test server");

        let response = server
            .get("/api/v1/stats")
            .add_header(
                HeaderName::from_static("authorization"),
                HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
            )
            .await;

        response.assert_status_ok();

        let json: Value = response.json();
        assert!(json["download"].is_object());
        assert!(json["upload"].is_object());
        assert!(json["ping"].is_object());

        // Check structure
        assert!(json["download"]["min"].is_object());
        assert!(json["download"]["avg"].is_object());
        assert!(json["download"]["max"].is_object());

        // Check it has bits and human fields
        assert!(json["download"]["avg"]["bits"].is_number());
        assert!(json["download"]["avg"]["human"].is_string());
    }

    #[tokio::test]
    async fn test_v1_results_pagination() {
        let state = setup_test_state().await;

        let token = create_test_token(&state, "results:read").await;

        // Create multiple results
        for _ in 0..10 {
            create_test_result(&state).await;
        }

        let app = create_app(state).await;
        let server = TestServer::new(app).expect("Failed to create test server");

        // Test pagination parameters
        let response = server
            .get("/api/v1/results?page=1&per_page=5")
            .add_header(
                HeaderName::from_static("authorization"),
                HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
            )
            .await;

        response.assert_status_ok();

        let json: Value = response.json();
        assert_eq!(json["page"], 1);
        assert_eq!(json["per_page"], 5);
        assert_eq!(json["data"].as_array().unwrap().len(), 5);
    }

    #[tokio::test]
    async fn test_invalid_bearer_token_returns_401() {
        let server = setup_test_app().await;

        let response = server
            .get("/api/v1/results")
            .add_header(
                HeaderName::from_static("authorization"),
                HeaderValue::from_str("Bearer invalid-token-123").unwrap(),
            )
            .await;

        response.assert_status(axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_case_insensitive_bearer_token() {
        let state = setup_test_state().await;

        let token = create_test_token(&state, "results:read").await;

        let app = create_app(state).await;
        let server = TestServer::new(app).expect("Failed to create test server");

        // Test with lowercase "bearer"
        let response = server
            .get("/api/v1/results")
            .add_header(
                HeaderName::from_static("authorization"),
                HeaderValue::from_str(&format!("bearer {}", token)).unwrap(),
            )
            .await;

        response.assert_status_ok();
    }

    #[tokio::test]
    async fn test_token_without_required_ability_is_denied() {
        let state = setup_test_state().await;

        // Create token without results:read ability
        let token = create_test_token(&state, "speedtests:run").await;

        let app = create_app(state).await;
        let server = TestServer::new(app).expect("Failed to create test server");

        let response = server
            .get("/api/v1/results")
            .add_header(
                HeaderName::from_static("authorization"),
                HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
            )
            .await;

        // Should be unauthorized because token doesn't have results:read ability
        response.assert_status(axum::http::StatusCode::UNAUTHORIZED);
    }
}
