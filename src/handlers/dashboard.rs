use crate::{AppState, db::Database, models::Result as SpeedTestResult};
use askama_axum::Template;
use axum::extract::{Query, State};
use serde::Deserialize;

#[derive(Template)]
#[template(path = "pages/dashboard.html")]
pub struct HomeDashboardTemplate {
    pub latest_results: Vec<SpeedTestResult>,
    pub stats: DashboardStats,
    pub time_range: String,
}

pub struct DashboardStats {
    pub total_tests: i64,
    pub avg_download: f64,
    pub avg_upload: f64,
    pub avg_ping: f64,
    pub chart_data: Vec<ChartDataPoint>,
}

pub struct ChartDataPoint {
    pub timestamp: String,
    pub download: f64,
    pub upload: f64,
    pub ping: f64,
}

#[derive(Deserialize)]
pub struct TimeRangeQuery {
    #[serde(default = "default_time_range")]
    range: String,
}

fn default_time_range() -> String {
    "24h".to_string()
}
pub async fn home_dashboard(
    State(state): State<AppState>,
    Query(params): Query<TimeRangeQuery>,
) -> HomeDashboardTemplate {
    let hours_ago = match params.range.as_str() {
        "week" => 24 * 7,
        "month" => 24 * 30,
        _ => 24, // default to 24h
    };

    // Use local time since database stores timestamps in local timezone
    let time_cutoff = (chrono::Local::now() - chrono::Duration::hours(hours_ago)).naive_local();

    let (latest_results, stats) = match &state.db {
        #[cfg(feature = "sqlite")]
        Database::Sqlite(pool) => {
            let latest = sqlx::query_as::<_, SpeedTestResult>(
                "SELECT * FROM results ORDER BY created_at DESC LIMIT 5",
            )
            .fetch_all(pool)
            .await
            .unwrap_or_default();

            let chart_results: Vec<SpeedTestResult> = sqlx::query_as(
                "SELECT * FROM results WHERE created_at >= ? AND status = ? ORDER BY created_at ASC"
            )
            .bind(time_cutoff)
            .bind("completed")
            .fetch_all(pool)
            .await
            .unwrap_or_default();

            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM results WHERE created_at >= ? AND status = ?",
            )
            .bind(time_cutoff)
            .bind("completed")
            .fetch_one(pool)
            .await
            .unwrap_or(0);

            let avg_download: Option<f64> = sqlx::query_scalar(
                "SELECT AVG(download) FROM results WHERE download IS NOT NULL AND created_at >= ? AND status = ?"
            )
            .bind(time_cutoff)
            .bind("completed")
            .fetch_one(pool)
            .await
            .ok();

            let avg_upload: Option<f64> = sqlx::query_scalar(
                "SELECT AVG(upload) FROM results WHERE upload IS NOT NULL AND created_at >= ? AND status = ?"
            )
            .bind(time_cutoff)
            .bind("completed")
            .fetch_one(pool)
            .await
            .ok();

            let avg_ping: Option<f64> = sqlx::query_scalar(
                "SELECT AVG(ping) FROM results WHERE ping IS NOT NULL AND created_at >= ? AND status = ?"
            )
            .bind(time_cutoff)
            .bind("completed")
            .fetch_one(pool)
            .await
            .ok();

            let chart_data = chart_results
                .iter()
                .map(|r| ChartDataPoint {
                    timestamp: r.created_at.format("%Y-%m-%d %H:%M").to_string(),
                    download: r.download_mbps(),
                    upload: r.upload_mbps(),
                    ping: r.ping.unwrap_or(0.0),
                })
                .collect();

            let stats = DashboardStats {
                total_tests: total,
                avg_download: avg_download.unwrap_or(0.0) * 8.0 / 1_000_000.0,
                avg_upload: avg_upload.unwrap_or(0.0) * 8.0 / 1_000_000.0,
                avg_ping: avg_ping.unwrap_or(0.0),
                chart_data,
            };

            (latest, stats)
        }
        #[cfg(feature = "mysql")]
        Database::MySql(pool) => {
            let latest = sqlx::query_as::<_, SpeedTestResult>(
                "SELECT * FROM results ORDER BY created_at DESC LIMIT 5",
            )
            .fetch_all(pool)
            .await
            .unwrap_or_default();

            let chart_results: Vec<SpeedTestResult> = sqlx::query_as(
                "SELECT * FROM results WHERE created_at >= ? AND status = ? ORDER BY created_at ASC"
            )
            .bind(time_cutoff)
            .bind("completed")
            .fetch_all(pool)
            .await
            .unwrap_or_default();

            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM results WHERE created_at >= ? AND status = ?",
            )
            .bind(time_cutoff)
            .bind("completed")
            .fetch_one(pool)
            .await
            .unwrap_or(0);

            let avg_download: Option<f64> = sqlx::query_scalar(
                "SELECT AVG(download) FROM results WHERE download IS NOT NULL AND created_at >= ? AND status = ?"
            )
            .bind(time_cutoff)
            .bind("completed")
            .fetch_one(pool)
            .await
            .ok();

            let avg_upload: Option<f64> = sqlx::query_scalar(
                "SELECT AVG(upload) FROM results WHERE upload IS NOT NULL AND created_at >= ? AND status = ?"
            )
            .bind(time_cutoff)
            .bind("completed")
            .fetch_one(pool)
            .await
            .ok();

            let avg_ping: Option<f64> = sqlx::query_scalar(
                "SELECT AVG(ping) FROM results WHERE ping IS NOT NULL AND created_at >= ? AND status = ?"
            )
            .bind(time_cutoff)
            .bind("completed")
            .fetch_one(pool)
            .await
            .ok();

            let chart_data = chart_results
                .iter()
                .map(|r| ChartDataPoint {
                    timestamp: r.created_at.format("%Y-%m-%d %H:%M").to_string(),
                    download: r.download_mbps(),
                    upload: r.upload_mbps(),
                    ping: r.ping.unwrap_or(0.0),
                })
                .collect();

            let stats = DashboardStats {
                total_tests: total,
                avg_download: avg_download.unwrap_or(0.0) * 8.0 / 1_000_000.0,
                avg_upload: avg_upload.unwrap_or(0.0) * 8.0 / 1_000_000.0,
                avg_ping: avg_ping.unwrap_or(0.0),
                chart_data,
            };

            (latest, stats)
        }
        #[cfg(feature = "postgres")]
        Database::Postgres(pool) => {
            let latest = sqlx::query_as::<_, SpeedTestResult>(
                "SELECT * FROM results ORDER BY created_at DESC LIMIT 5",
            )
            .fetch_all(pool)
            .await
            .unwrap_or_default();

            let chart_results: Vec<SpeedTestResult> = sqlx::query_as(
                "SELECT * FROM results WHERE created_at >= $1 AND status = $2 ORDER BY created_at ASC"
            )
            .bind(time_cutoff)
            .bind("completed")
            .fetch_all(pool)
            .await
            .unwrap_or_default();

            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM results WHERE created_at >= $1 AND status = $2",
            )
            .bind(time_cutoff)
            .bind("completed")
            .fetch_one(pool)
            .await
            .unwrap_or(0);

            let avg_download: Option<f64> = sqlx::query_scalar(
                "SELECT AVG(download) FROM results WHERE download IS NOT NULL AND created_at >= $1 AND status = $2"
            )
            .bind(time_cutoff)
            .bind("completed")
            .fetch_one(pool)
            .await
            .ok();

            let avg_upload: Option<f64> = sqlx::query_scalar(
                "SELECT AVG(upload) FROM results WHERE upload IS NOT NULL AND created_at >= $1 AND status = $2"
            )
            .bind(time_cutoff)
            .bind("completed")
            .fetch_one(pool)
            .await
            .ok();

            let avg_ping: Option<f64> = sqlx::query_scalar(
                "SELECT AVG(ping) FROM results WHERE ping IS NOT NULL AND created_at >= $1 AND status = $2"
            )
            .bind(time_cutoff)
            .bind("completed")
            .fetch_one(pool)
            .await
            .ok();

            let chart_data = chart_results
                .iter()
                .map(|r| ChartDataPoint {
                    timestamp: r.created_at.format("%Y-%m-%d %H:%M").to_string(),
                    download: r.download_mbps(),
                    upload: r.upload_mbps(),
                    ping: r.ping.unwrap_or(0.0),
                })
                .collect();

            let stats = DashboardStats {
                total_tests: total,
                avg_download: avg_download.unwrap_or(0.0) * 8.0 / 1_000_000.0,
                avg_upload: avg_upload.unwrap_or(0.0) * 8.0 / 1_000_000.0,
                avg_ping: avg_ping.unwrap_or(0.0),
                chart_data,
            };

            (latest, stats)
        }
    };

    HomeDashboardTemplate {
        latest_results,
        stats,
        time_range: params.range.clone(),
    }
}
