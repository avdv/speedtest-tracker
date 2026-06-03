use crate::{AppState, db::Database, models::Result as SpeedTestResult};
use askama_axum::Template;
use axum::{
    Form,
    extract::{Query, State},
    response::{IntoResponse, Redirect},
};
use serde::Deserialize;

#[derive(Template)]
#[template(path = "pages/results.html")]
pub struct ResultsListTemplate {
    results: Vec<SpeedTestResult>,
    page: i64,
    per_page: i64,
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
    Query(params): Query<Pagination>,
) -> ResultsListTemplate {
    let offset = (params.page - 1) * params.per_page;

    let results = match &state.db {
        #[cfg(feature = "sqlite")]
        Database::Sqlite(pool) => sqlx::query_as::<_, SpeedTestResult>(
            "SELECT * FROM results ORDER BY created_at DESC LIMIT ? OFFSET ?",
        )
        .bind(params.per_page)
        .bind(offset)
        .fetch_all(pool)
        .await
        .unwrap_or_default(),
        #[cfg(feature = "mysql")]
        Database::MySql(pool) => sqlx::query_as::<_, SpeedTestResult>(
            "SELECT * FROM results ORDER BY created_at DESC LIMIT ? OFFSET ?",
        )
        .bind(params.per_page)
        .bind(offset)
        .fetch_all(pool)
        .await
        .unwrap_or_default(),
        #[cfg(feature = "postgres")]
        Database::Postgres(pool) => sqlx::query_as::<_, SpeedTestResult>(
            "SELECT * FROM results ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(params.per_page)
        .bind(offset)
        .fetch_all(pool)
        .await
        .unwrap_or_default(),
    };

    ResultsListTemplate {
        results,
        page: params.page,
        per_page: params.per_page,
    }
}

#[derive(Deserialize)]
pub struct DeleteResultsForm {
    ids: String,
}

pub async fn delete_results(
    State(state): State<AppState>,
    Form(form): Form<DeleteResultsForm>,
) -> impl IntoResponse {
    let ids: Vec<i64> = form
        .ids
        .split(',')
        .filter_map(|s| s.trim().parse::<i64>().ok())
        .collect();

    if ids.is_empty() {
        return Redirect::to("/admin/results");
    }

    match &state.db {
        #[cfg(feature = "sqlite")]
        Database::Sqlite(pool) => {
            for id in ids {
                let _ = sqlx::query("DELETE FROM results WHERE id = ?")
                    .bind(id)
                    .execute(pool)
                    .await;
            }
        }
        #[cfg(feature = "mysql")]
        Database::MySql(pool) => {
            for id in ids {
                let _ = sqlx::query("DELETE FROM results WHERE id = ?")
                    .bind(id)
                    .execute(pool)
                    .await;
            }
        }
        #[cfg(feature = "postgres")]
        Database::Postgres(pool) => {
            for id in ids {
                let _ = sqlx::query("DELETE FROM results WHERE id = $1")
                    .bind(id)
                    .execute(pool)
                    .await;
            }
        }
    }

    Redirect::to("/admin/results")
}
