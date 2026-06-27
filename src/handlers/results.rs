use crate::error::{AppError, HtmlTemplate};
use crate::locale_middleware::Locale;
use crate::{db::Database, filters, models::Result as SpeedTestResult, AppState};
use askama::Template;
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
    Form,
};
use serde::Deserialize;

#[derive(Template)]
#[template(path = "pages/results.html")]
pub struct ResultsListTemplate {
    locale: String,
    results: Vec<SpeedTestResult>,
    page: i64,
    per_page: i64,
    total_results: i64,
    total_pages: i64,
    is_authenticated: bool,
}

#[derive(Deserialize)]
pub struct Pagination {
    #[serde(default = "default_page")]
    page: i64,
    #[serde(default = "default_per_page")]
    per_page: i64,
}

fn default_page() -> i64 {
    1
}
fn default_per_page() -> i64 {
    25
}

pub async fn results_list(
    State(state): State<AppState>,
    locale: Locale,
    Query(params): Query<Pagination>,
) -> Result<impl IntoResponse, AppError> {
    let offset = (params.page - 1) * params.per_page;

    // Get total count
    let total_results: i64 = match &state.db {
        #[cfg(feature = "sqlite")]
        Database::Sqlite(pool) => {
            sqlx::query_scalar("SELECT COUNT(*) FROM results")
                .fetch_one(pool)
                .await?
        }
        #[cfg(feature = "mysql")]
        Database::MySql(pool) => {
            sqlx::query_scalar("SELECT COUNT(*) FROM results")
                .fetch_one(pool)
                .await?
        }
        #[cfg(feature = "postgres")]
        Database::Postgres(pool) => {
            sqlx::query_scalar("SELECT COUNT(*) FROM results")
                .fetch_one(pool)
                .await?
        }
    };

    let total_pages = if total_results > 0 {
        (total_results + params.per_page - 1) / params.per_page
    } else {
        1
    };

    let results = match &state.db {
        #[cfg(feature = "sqlite")]
        Database::Sqlite(pool) => {
            sqlx::query_as::<_, SpeedTestResult>(
                "SELECT * FROM results ORDER BY created_at DESC LIMIT ? OFFSET ?",
            )
            .bind(params.per_page)
            .bind(offset)
            .fetch_all(pool)
            .await?
        }
        #[cfg(feature = "mysql")]
        Database::MySql(pool) => {
            sqlx::query_as::<_, SpeedTestResult>(
                "SELECT * FROM results ORDER BY created_at DESC LIMIT ? OFFSET ?",
            )
            .bind(params.per_page)
            .bind(offset)
            .fetch_all(pool)
            .await?
        }
        #[cfg(feature = "postgres")]
        Database::Postgres(pool) => {
            sqlx::query_as::<_, SpeedTestResult>(
                "SELECT * FROM results ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            )
            .bind(params.per_page)
            .bind(offset)
            .fetch_all(pool)
            .await?
        }
    };

    Ok(HtmlTemplate(ResultsListTemplate {
        locale: locale.0,
        results,
        page: params.page,
        per_page: params.per_page,
        total_results,
        total_pages,
        is_authenticated: true,
    }))
}

#[derive(Deserialize)]
pub struct DeleteResultsForm {
    ids: String,
}

pub async fn delete_results(
    State(state): State<AppState>,
    Form(form): Form<DeleteResultsForm>,
) -> Result<impl IntoResponse, AppError> {
    let ids: Vec<i64> = form
        .ids
        .split(',')
        .filter_map(|s| s.trim().parse::<i64>().ok())
        .collect();

    if ids.is_empty() {
        return Ok(Redirect::to("/admin/results"));
    }

    match &state.db {
        #[cfg(feature = "sqlite")]
        Database::Sqlite(pool) => {
            for id in ids {
                sqlx::query("DELETE FROM results WHERE id = ?")
                    .bind(id)
                    .execute(pool)
                    .await?;
            }
        }
        #[cfg(feature = "mysql")]
        Database::MySql(pool) => {
            for id in ids {
                sqlx::query("DELETE FROM results WHERE id = ?")
                    .bind(id)
                    .execute(pool)
                    .await?;
            }
        }
        #[cfg(feature = "postgres")]
        Database::Postgres(pool) => {
            for id in ids {
                sqlx::query("DELETE FROM results WHERE id = $1")
                    .bind(id)
                    .execute(pool)
                    .await?;
            }
        }
    }

    Ok(Redirect::to("/admin/results"))
}
