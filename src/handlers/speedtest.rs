use crate::{AppState, db::Database, models::Result as SpeedTestResult};
use askama_axum::Template;
use axum::{
    Form, Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Template)]
#[template(path = "pages/run-test.html")]
pub struct RunTestTemplate {
    pub servers: Vec<crate::api::OoklaServer>,
}

pub async fn run_test_page() -> RunTestTemplate {
    // Fetch server list (can be cached in production)
    let servers = crate::api::fetch_ookla_servers().await.unwrap_or_default();

    RunTestTemplate {
        servers: servers.into_iter().take(50).collect(), // Limit to top 50
    }
}

#[derive(serde::Deserialize)]
pub struct RunTestForm {
    server_id: Option<String>,
}

pub async fn run_test_execute(
    State(state): State<AppState>,
    Form(form): Form<RunTestForm>,
) -> Response {
    // Parse server_id
    let server_id = form
        .server_id
        .and_then(|s| if s.is_empty() { None } else { Some(s) })
        .and_then(|s| s.parse::<i64>().ok());

    tracing::info!("Manual speedtest requested with server_id: {:?}", server_id);

    // Run speedtest
    let result = match crate::speedtest::run_speedtest(server_id).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Speedtest execution failed: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": e
                })),
            )
                .into_response();
        }
    };

    // Save to database
    let result_id = match crate::speedtest::save_result(&state.db, result, false).await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("Failed to save speedtest result: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Test completed but failed to save: {}", e)
                })),
            )
                .into_response();
        }
    };

    // Fetch the saved result to return
    let saved_result = match &state.db {
        #[cfg(feature = "sqlite")]
        Database::Sqlite(pool) => {
            sqlx::query_as::<_, SpeedTestResult>("SELECT * FROM results WHERE id = ?")
                .bind(result_id)
                .fetch_one(pool)
                .await
        }
        #[cfg(feature = "mysql")]
        Database::MySql(pool) => {
            sqlx::query_as::<_, SpeedTestResult>("SELECT * FROM results WHERE id = ?")
                .bind(result_id)
                .fetch_one(pool)
                .await
        }
        #[cfg(feature = "postgres")]
        Database::Postgres(pool) => {
            sqlx::query_as::<_, SpeedTestResult>("SELECT * FROM results WHERE id = $1")
                .bind(result_id)
                .fetch_one(pool)
                .await
        }
    };

    match saved_result {
        Ok(result) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "id": result.id,
                "download_mbps": format!("{:.2}", result.download_mbps()),
                "upload_mbps": format!("{:.2}", result.upload_mbps()),
                "ping": format!("{:.1}", result.ping.unwrap_or(0.0))
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch saved result: {}", e);
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "id": result_id,
                    "message": "Test completed successfully"
                })),
            )
                .into_response()
        }
    }
}
