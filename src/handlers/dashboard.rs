use crate::{AppState, db::Database, filters, models::Result as SpeedTestResult};
use crate::locale_middleware::Locale;
use askama::Template;
use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse, Response},
};
use serde::Deserialize;
use chrono::{Local, NaiveDateTime};
use std::env;
use std::str::FromStr;

#[derive(Template)]
#[template(path = "pages/dashboard.html")]
pub struct HomeDashboardTemplate {
    pub locale: String,
    pub latest_results: Vec<SpeedTestResult>,
    pub stats: DashboardStats,
    pub time_range: String,
    pub next_speedtest: Option<NaiveDateTime>,
    pub is_authenticated: bool,
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

fn get_next_scheduled_test() -> Option<NaiveDateTime> {
    let schedule_expr = env::var("SPEEDTEST_SCHEDULE").ok()?;
    
    if schedule_expr.is_empty() {
        return None;
    }

    // Convert 5-field cron to 6-field (add seconds at start) if needed
    let schedule_expr = if schedule_expr.split_whitespace().count() == 5 {
        format!("0 {}", schedule_expr)
    } else {
        schedule_expr
    };

    // Parse the cron expression
    let schedule = match cron::Schedule::from_str(&schedule_expr) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("Failed to parse cron expression '{}': {}", schedule_expr, e);
            return None;
        }
    };

    // Get the next run time and convert to NaiveDateTime
    schedule.upcoming(Local).take(1).next().map(|next| next.naive_local())
}

pub async fn home_dashboard(
    State(state): State<AppState>,
    locale: Locale,
    session: tower_sessions::Session,
    Query(params): Query<TimeRangeQuery>,
) -> Response {
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
                    timestamp: r.created_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
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
                    timestamp: r.created_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
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
                    timestamp: r.created_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
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

    let next_speedtest = get_next_scheduled_test();
    let is_authenticated = crate::session::get_user_id(session).await.is_some();

    let template = HomeDashboardTemplate {
        locale: locale.0,
        latest_results,
        stats,
        time_range: params.range.clone(),
        next_speedtest,
        is_authenticated,
    };
    
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(err) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}
