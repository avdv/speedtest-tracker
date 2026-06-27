use crate::error::{AppError, HtmlTemplate};
use crate::locale_middleware::Locale;
use crate::{db::Database, filters, models::Result as SpeedTestResult, AppState};
use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Form, Json,
};

#[derive(Template)]
#[template(path = "pages/run-test.html")]
pub struct RunTestTemplate {
    pub locale: String,
    pub servers: Vec<crate::api::OoklaServer>,
    pub is_authenticated: bool,
}

#[axum::debug_handler]
pub async fn run_test_page(locale: Locale) -> Result<impl IntoResponse, AppError> {
    let servers = crate::api::fetch_ookla_servers().await.unwrap_or_default();

    Ok(HtmlTemplate(RunTestTemplate {
        locale: locale.0,
        servers: servers.into_iter().take(50).collect(),
        is_authenticated: true,
    }))
}

#[derive(serde::Deserialize)]
pub struct RunTestForm {
    server_id: Option<String>,
}

pub async fn run_test_execute(
    State(state): State<AppState>,
    Form(form): Form<RunTestForm>,
) -> Result<Response, AppError> {
    let server_id = form
        .server_id
        .and_then(|s| if s.is_empty() { None } else { Some(s) })
        .and_then(|s| s.parse::<i64>().ok());

    tracing::info!("Manual speedtest requested with server_id: {:?}", server_id);

    let result = crate::speedtest::run_speedtest(server_id)
        .await
        .map_err(|e| {
            tracing::error!("Speedtest execution failed: {}", e);
            AppError::from(anyhow::anyhow!("{}", e))
        })?;

    let result_id = crate::speedtest::save_result(&state.db, result, false)
        .await
        .map_err(|e| {
            tracing::error!("Failed to save speedtest result: {}", e);
            AppError::from(anyhow::anyhow!("Test completed but failed to save: {}", e))
        })?;

    let saved_result = match &state.db {
        #[cfg(feature = "sqlite")]
        Database::Sqlite(pool) => {
            sqlx::query_as::<_, SpeedTestResult>("SELECT * FROM results WHERE id = ?")
                .bind(result_id)
                .fetch_one(pool)
                .await?
        }
        #[cfg(feature = "mysql")]
        Database::MySql(pool) => {
            sqlx::query_as::<_, SpeedTestResult>("SELECT * FROM results WHERE id = ?")
                .bind(result_id)
                .fetch_one(pool)
                .await?
        }
        #[cfg(feature = "postgres")]
        Database::Postgres(pool) => {
            sqlx::query_as::<_, SpeedTestResult>("SELECT * FROM results WHERE id = $1")
                .bind(result_id)
                .fetch_one(pool)
                .await?
        }
    };

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "id": saved_result.id,
            "download_mbps": format!("{:.2}", saved_result.download_mbps()),
            "upload_mbps": format!("{:.2}", saved_result.upload_mbps()),
            "ping": format!("{:.1}", saved_result.ping.unwrap_or(0.0))
        })),
    )
        .into_response())
}
