use crate::{filters, AppState, db::Database, models::Result as SpeedTestResult};
use askama_axum::Template;
use axum::extract::State;

#[derive(Template)]
#[template(path = "pages/admin.html")]
pub struct AdminDashboardTemplate {
    pub stats: AdminStats,
    pub latest_result: Option<SpeedTestResult>,
}

pub struct AdminStats {
    pub total_tests: i64,
    pub avg_download: f64,
    pub avg_upload: f64,
    pub avg_ping: f64,
}
pub async fn admin_dashboard(State(state): State<AppState>) -> AdminDashboardTemplate {
    let (latest_result, stats) = match &state.db {
        #[cfg(feature = "sqlite")]
        Database::Sqlite(pool) => {
            let latest = sqlx::query_as::<_, SpeedTestResult>(
                "SELECT * FROM results ORDER BY created_at DESC LIMIT 1",
            )
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM results")
                .fetch_one(pool)
                .await
                .unwrap_or(0);

            let avg_download: Option<f64> =
                sqlx::query_scalar("SELECT AVG(download) FROM results WHERE download IS NOT NULL")
                    .fetch_one(pool)
                    .await
                    .ok();

            let avg_upload: Option<f64> =
                sqlx::query_scalar("SELECT AVG(upload) FROM results WHERE upload IS NOT NULL")
                    .fetch_one(pool)
                    .await
                    .ok();

            let avg_ping: Option<f64> =
                sqlx::query_scalar("SELECT AVG(ping) FROM results WHERE ping IS NOT NULL")
                    .fetch_one(pool)
                    .await
                    .ok();

            let stats = AdminStats {
                total_tests: total,
                avg_download: avg_download.unwrap_or(0.0) * 8.0 / 1_000_000.0,
                avg_upload: avg_upload.unwrap_or(0.0) * 8.0 / 1_000_000.0,
                avg_ping: avg_ping.unwrap_or(0.0),
            };

            (latest, stats)
        }
        #[cfg(feature = "mysql")]
        Database::MySql(pool) => {
            let latest = sqlx::query_as::<_, SpeedTestResult>(
                "SELECT * FROM results ORDER BY created_at DESC LIMIT 1",
            )
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM results")
                .fetch_one(pool)
                .await
                .unwrap_or(0);

            let avg_download: Option<f64> =
                sqlx::query_scalar("SELECT AVG(download) FROM results WHERE download IS NOT NULL")
                    .fetch_one(pool)
                    .await
                    .ok();

            let avg_upload: Option<f64> =
                sqlx::query_scalar("SELECT AVG(upload) FROM results WHERE upload IS NOT NULL")
                    .fetch_one(pool)
                    .await
                    .ok();

            let avg_ping: Option<f64> =
                sqlx::query_scalar("SELECT AVG(ping) FROM results WHERE ping IS NOT NULL")
                    .fetch_one(pool)
                    .await
                    .ok();

            let stats = AdminStats {
                total_tests: total,
                avg_download: avg_download.unwrap_or(0.0) * 8.0 / 1_000_000.0,
                avg_upload: avg_upload.unwrap_or(0.0) * 8.0 / 1_000_000.0,
                avg_ping: avg_ping.unwrap_or(0.0),
            };

            (latest, stats)
        }
        #[cfg(feature = "postgres")]
        Database::Postgres(pool) => {
            let latest = sqlx::query_as::<_, SpeedTestResult>(
                "SELECT * FROM results ORDER BY created_at DESC LIMIT 1",
            )
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM results")
                .fetch_one(pool)
                .await
                .unwrap_or(0);

            let avg_download: Option<f64> =
                sqlx::query_scalar("SELECT AVG(download) FROM results WHERE download IS NOT NULL")
                    .fetch_one(pool)
                    .await
                    .ok();

            let avg_upload: Option<f64> =
                sqlx::query_scalar("SELECT AVG(upload) FROM results WHERE upload IS NOT NULL")
                    .fetch_one(pool)
                    .await
                    .ok();

            let avg_ping: Option<f64> =
                sqlx::query_scalar("SELECT AVG(ping) FROM results WHERE ping IS NOT NULL")
                    .fetch_one(pool)
                    .await
                    .ok();

            let stats = AdminStats {
                total_tests: total,
                avg_download: avg_download.unwrap_or(0.0) * 8.0 / 1_000_000.0,
                avg_upload: avg_upload.unwrap_or(0.0) * 8.0 / 1_000_000.0,
                avg_ping: avg_ping.unwrap_or(0.0),
            };

            (latest, stats)
        }
    };

    AdminDashboardTemplate {
        stats,
        latest_result,
    }
}
