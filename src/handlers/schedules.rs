use crate::error::{AppError, HtmlTemplate};
use crate::locale_middleware::Locale;
use crate::{db::Database, filters, models::Schedule, AppState};
use askama::Template;
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
    Form,
};
use serde::Deserialize;

#[derive(Template)]
#[template(path = "pages/schedules.html")]
pub struct SchedulesTemplate {
    pub locale: String,
    pub schedules: Vec<Schedule>,
    pub servers: Vec<crate::api::OoklaServer>,
    pub message: Option<String>,
    pub is_authenticated: bool,
}

pub async fn schedules_page(
    State(state): State<AppState>,
    locale: Locale,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<impl IntoResponse, AppError> {
    let message = if params.contains_key("created") {
        Some("Schedule created successfully!".to_string())
    } else if params.contains_key("updated") {
        Some("Schedule updated successfully!".to_string())
    } else if params.contains_key("deleted") {
        Some("Schedule deleted successfully!".to_string())
    } else if params.contains_key("error") {
        Some("An error occurred.".to_string())
    } else {
        None
    };

    let schedules = match &state.db {
        #[cfg(feature = "sqlite")]
        Database::Sqlite(pool) => {
            sqlx::query_as::<_, Schedule>("SELECT * FROM schedules ORDER BY created_at DESC")
                .fetch_all(pool)
                .await?
        }
        #[cfg(feature = "mysql")]
        Database::MySql(pool) => {
            sqlx::query_as::<_, Schedule>("SELECT * FROM schedules ORDER BY created_at DESC")
                .fetch_all(pool)
                .await?
        }
        #[cfg(feature = "postgres")]
        Database::Postgres(pool) => {
            sqlx::query_as::<_, Schedule>("SELECT * FROM schedules ORDER BY created_at DESC")
                .fetch_all(pool)
                .await?
        }
    };

    let servers = crate::api::fetch_ookla_servers().await.unwrap_or_default();

    let template = SchedulesTemplate {
        locale: locale.0,
        schedules,
        servers: servers.into_iter().take(100).collect(),
        message,
        is_authenticated: true,
    };

    Ok(HtmlTemplate(template))
}

#[derive(Deserialize)]
pub struct CreateScheduleForm {
    name: String,
    cron: String,
    server_ids: Option<String>,
    enabled: Option<String>,
}

pub async fn create_schedule(
    State(state): State<AppState>,
    Form(form): Form<CreateScheduleForm>,
) -> Result<impl IntoResponse, AppError> {
    let enabled = form.enabled.is_some();
    let server_ids = form.server_ids.filter(|s| !s.trim().is_empty());

    let success = match &state.db {
        #[cfg(feature = "sqlite")]
        Database::Sqlite(pool) => sqlx::query(
            "INSERT INTO schedules (name, cron, server_ids, enabled, created_at, updated_at)
                 VALUES (?, ?, ?, ?, datetime('now'), datetime('now'))",
        )
        .bind(&form.name)
        .bind(&form.cron)
        .bind(&server_ids)
        .bind(enabled)
        .execute(pool)
        .await
        .is_ok(),
        #[cfg(feature = "mysql")]
        Database::MySql(pool) => sqlx::query(
            "INSERT INTO schedules (name, cron, server_ids, enabled, created_at, updated_at)
                 VALUES (?, ?, ?, ?, NOW(), NOW())",
        )
        .bind(&form.name)
        .bind(&form.cron)
        .bind(&server_ids)
        .bind(enabled)
        .execute(pool)
        .await
        .is_ok(),
        #[cfg(feature = "postgres")]
        Database::Postgres(pool) => sqlx::query(
            "INSERT INTO schedules (name, cron, server_ids, enabled, created_at, updated_at)
                 VALUES ($1, $2, $3, $4, NOW(), NOW())",
        )
        .bind(&form.name)
        .bind(&form.cron)
        .bind(&server_ids)
        .bind(enabled)
        .execute(pool)
        .await
        .is_ok(),
    };

    if success {
        Ok(Redirect::to("/admin/schedules?created=1"))
    } else {
        Ok(Redirect::to("/admin/schedules?error=1"))
    }
}

#[derive(Deserialize)]
pub struct DeleteScheduleForm {
    id: i64,
}

pub async fn delete_schedule(
    State(state): State<AppState>,
    Form(form): Form<DeleteScheduleForm>,
) -> Result<impl IntoResponse, AppError> {
    let success = match &state.db {
        #[cfg(feature = "sqlite")]
        Database::Sqlite(pool) => sqlx::query("DELETE FROM schedules WHERE id = ?")
            .bind(form.id)
            .execute(pool)
            .await
            .is_ok(),
        #[cfg(feature = "mysql")]
        Database::MySql(pool) => sqlx::query("DELETE FROM schedules WHERE id = ?")
            .bind(form.id)
            .execute(pool)
            .await
            .is_ok(),
        #[cfg(feature = "postgres")]
        Database::Postgres(pool) => sqlx::query("DELETE FROM schedules WHERE id = $1")
            .bind(form.id)
            .execute(pool)
            .await
            .is_ok(),
    };

    if success {
        Ok(Redirect::to("/admin/schedules?deleted=1"))
    } else {
        Ok(Redirect::to("/admin/schedules?error=1"))
    }
}

#[derive(Deserialize)]
pub struct ToggleScheduleForm {
    id: i64,
}

pub async fn toggle_schedule(
    State(state): State<AppState>,
    Form(form): Form<ToggleScheduleForm>,
) -> Result<impl IntoResponse, AppError> {
    let success = match &state.db {
        #[cfg(feature = "sqlite")]
        Database::Sqlite(pool) => sqlx::query(
            "UPDATE schedules SET enabled = NOT enabled, updated_at = datetime('now') WHERE id = ?",
        )
        .bind(form.id)
        .execute(pool)
        .await
        .is_ok(),
        #[cfg(feature = "mysql")]
        Database::MySql(pool) => sqlx::query(
            "UPDATE schedules SET enabled = NOT enabled, updated_at = NOW() WHERE id = ?",
        )
        .bind(form.id)
        .execute(pool)
        .await
        .is_ok(),
        #[cfg(feature = "postgres")]
        Database::Postgres(pool) => sqlx::query(
            "UPDATE schedules SET enabled = NOT enabled, updated_at = NOW() WHERE id = $1",
        )
        .bind(form.id)
        .execute(pool)
        .await
        .is_ok(),
    };

    if success {
        Ok(Redirect::to("/admin/schedules?updated=1"))
    } else {
        Ok(Redirect::to("/admin/schedules?error=1"))
    }
}
