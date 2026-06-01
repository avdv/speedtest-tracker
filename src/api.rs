use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use crate::{models::Result as SpeedTestResult, db::Database, AppState};

#[derive(Serialize)]
pub struct ApiResponse<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    pub message: String,
}

#[derive(Serialize)]
pub struct ResultResponse {
    pub id: i64,
    pub service: String,
    pub ping: Option<f64>,
    pub download: Option<i64>,
    pub upload: Option<i64>,
    pub comments: Option<String>,
    pub data: Option<serde_json::Value>,
    pub status: String,
    pub scheduled: bool,
    pub healthy: Option<bool>,
    pub download_bits: Option<i64>,
    pub upload_bits: Option<i64>,
    pub download_bytes: Option<i64>,
    pub upload_bytes: Option<i64>,
    pub download_bits_human: Option<String>,
    pub upload_bits_human: Option<String>,
    pub download_bytes_human: Option<String>,
    pub upload_bytes_human: Option<String>,
    pub dispatched_by: Option<i64>,
    pub benchmarks: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<SpeedTestResult> for ResultResponse {
    fn from(result: SpeedTestResult) -> Self {
        // Parse the data JSON if available
        let data_json = result.data.as_ref()
            .and_then(|d| serde_json::from_str(d).ok());
        
        // Calculate bits and bytes
        let download_bits = result.download.map(|d| d * 8);
        let upload_bits = result.upload.map(|u| u * 8);
        let download_bytes = result.download;
        let upload_bytes = result.upload;
        
        // Format human-readable values
        let download_bits_human = download_bits.map(|b| format_bits(b as f64));
        let upload_bits_human = upload_bits.map(|b| format_bits(b as f64));
        let download_bytes_human = download_bytes.map(|b| format_bytes(b as f64));
        let upload_bytes_human = upload_bytes.map(|b| format_bytes(b as f64));
        
        ResultResponse {
            id: result.id,
            service: result.service,
            ping: result.ping,
            download: result.download,
            upload: result.upload,
            comments: result.comments,
            data: data_json,
            status: result.status,
            scheduled: result.scheduled,
            healthy: None, // TODO: calculate from database if column exists
            download_bits,
            upload_bits,
            download_bytes,
            upload_bytes,
            download_bits_human,
            upload_bits_human,
            download_bytes_human,
            upload_bytes_human,
            dispatched_by: None, // TODO: fetch from database if column exists
            benchmarks: None, // TODO: fetch from database if column exists
            created_at: result.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: result.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

fn format_bits(bits: f64) -> String {
    let mbps = bits / 1_000_000.0;
    if mbps >= 1000.0 {
        format!("{:.2} Gbps", mbps / 1000.0)
    } else {
        format!("{:.2} Mbps", mbps)
    }
}

fn format_bytes(bytes: f64) -> String {
    if bytes >= 1_073_741_824.0 {
        format!("{:.0} GB", bytes / 1_073_741_824.0)
    } else if bytes >= 1_048_576.0 {
        format!("{:.0} MB", bytes / 1_048_576.0)
    } else if bytes >= 1024.0 {
        format!("{:.0} KB", bytes / 1024.0)
    } else {
        format!("{:.0} B", bytes)
    }
}

#[derive(Deserialize)]
pub struct ListQuery {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_per_page")]
    pub per_page: i64,
}

fn default_page() -> i64 { 1 }
fn default_per_page() -> i64 { 25 }

#[derive(Serialize)]
pub struct PaginatedResults {
    pub data: Vec<ResultResponse>,
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
}

// GET /api/healthcheck
pub async fn healthcheck() -> Json<ApiResponse<()>> {
    Json(ApiResponse {
        data: None,
        message: "Speedtest Tracker is running!".to_string(),
    })
}

// GET /api/speedtest/latest (legacy v0 endpoint)
pub async fn legacy_latest(State(state): State<AppState>) -> impl IntoResponse {
    let result = match &state.db {
        Database::Sqlite(pool) => {
            sqlx::query_as::<_, SpeedTestResult>(
                "SELECT * FROM results WHERE status IN ('completed', 'failed') ORDER BY created_at DESC LIMIT 1"
            )
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()
        },
        Database::MySql(pool) => {
            sqlx::query_as::<_, SpeedTestResult>(
                "SELECT * FROM results WHERE status IN ('completed', 'failed') ORDER BY created_at DESC LIMIT 1"
            )
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()
        },
        Database::Postgres(pool) => {
            sqlx::query_as::<_, SpeedTestResult>(
                "SELECT * FROM results WHERE status IN ('completed', 'failed') ORDER BY created_at DESC LIMIT 1"
            )
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()
        },
    };

    match result {
        Some(r) => {
            // Parse server info from data JSON if available
            let (server_id, server_host, server_name, result_url) = r.data.as_ref()
                .and_then(|d| serde_json::from_str::<serde_json::Value>(d).ok())
                .and_then(|json| {
                    let server = json.get("server")?;
                    let result = json.get("result")?;
                    Some((
                        server.get("id").and_then(|v| v.as_i64()),
                        server.get("host").and_then(|v| v.as_str().map(String::from)),
                        server.get("name").and_then(|v| v.as_str().map(String::from)),
                        result.get("url").and_then(|v| v.as_str().map(String::from)),
                    ))
                })
                .unwrap_or((None, None, None, None));

            let response = serde_json::json!({
                "message": "ok",
                "data": {
                    "id": r.id,
                    "ping": r.ping,
                    "download": r.download_mbps(),
                    "upload": r.upload_mbps(),
                    "server_id": server_id,
                    "server_host": server_host,
                    "server_name": server_name,
                    "url": result_url,
                    "scheduled": r.scheduled,
                    "failed": r.status == "failed",
                    "created_at": r.created_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
                    "updated_at": r.updated_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
                }
            });
            (StatusCode::OK, Json(response))
        },
        None => {
            let response = serde_json::json!({
                "message": "No results found."
            });
            (StatusCode::NOT_FOUND, Json(response))
        }
    }
}

// GET /api/v1/results
pub async fn list_results(
    State(state): State<AppState>,
    Query(params): Query<ListQuery>,
) -> Json<PaginatedResults> {
    let offset = (params.page - 1) * params.per_page;
    
    let (results, total) = match &state.db {
        Database::Sqlite(pool) => {
            let results = sqlx::query_as::<_, SpeedTestResult>(
                "SELECT * FROM results ORDER BY created_at DESC LIMIT ? OFFSET ?"
            )
            .bind(params.per_page)
            .bind(offset)
            .fetch_all(pool)
            .await
            .unwrap_or_default();
            
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM results")
                .fetch_one(pool)
                .await
                .unwrap_or(0);
            
            (results, total)
        },
        Database::MySql(pool) => {
            let results = sqlx::query_as::<_, SpeedTestResult>(
                "SELECT * FROM results ORDER BY created_at DESC LIMIT ? OFFSET ?"
            )
            .bind(params.per_page)
            .bind(offset)
            .fetch_all(pool)
            .await
            .unwrap_or_default();
            
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM results")
                .fetch_one(pool)
                .await
                .unwrap_or(0);
            
            (results, total)
        },
        Database::Postgres(pool) => {
            let results = sqlx::query_as::<_, SpeedTestResult>(
                "SELECT * FROM results ORDER BY created_at DESC LIMIT $1 OFFSET $2"
            )
            .bind(params.per_page)
            .bind(offset)
            .fetch_all(pool)
            .await
            .unwrap_or_default();
            
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM results")
                .fetch_one(pool)
                .await
                .unwrap_or(0);
            
            (results, total)
        },
    };

    Json(PaginatedResults {
        data: results.into_iter().map(Into::into).collect(),
        page: params.page,
        per_page: params.per_page,
        total,
    })
}

// GET /api/v1/results/latest
pub async fn latest_result(State(state): State<AppState>) -> impl IntoResponse {
    let result = match &state.db {
        Database::Sqlite(pool) => {
            sqlx::query_as::<_, SpeedTestResult>(
                "SELECT * FROM results ORDER BY created_at DESC LIMIT 1"
            )
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()
        },
        Database::MySql(pool) => {
            sqlx::query_as::<_, SpeedTestResult>(
                "SELECT * FROM results ORDER BY created_at DESC LIMIT 1"
            )
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()
        },
        Database::Postgres(pool) => {
            sqlx::query_as::<_, SpeedTestResult>(
                "SELECT * FROM results ORDER BY created_at DESC LIMIT 1"
            )
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()
        },
    };

    match result {
        Some(r) => {
            let response = ApiResponse {
                data: Some(ResultResponse::from(r)),
                message: "Success".to_string(),
            };
            (StatusCode::OK, Json(response))
        },
        None => {
            let response: ApiResponse<ResultResponse> = ApiResponse {
                data: None,
                message: "No results found.".to_string(),
            };
            (StatusCode::NOT_FOUND, Json(response))
        }
    }
}

// GET /api/v1/results/{id}
pub async fn get_result(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let result = match &state.db {
        Database::Sqlite(pool) => {
            sqlx::query_as::<_, SpeedTestResult>("SELECT * FROM results WHERE id = ?")
                .bind(id)
                .fetch_optional(pool)
                .await
                .ok()
                .flatten()
        },
        Database::MySql(pool) => {
            sqlx::query_as::<_, SpeedTestResult>("SELECT * FROM results WHERE id = ?")
                .bind(id)
                .fetch_optional(pool)
                .await
                .ok()
                .flatten()
        },
        Database::Postgres(pool) => {
            sqlx::query_as::<_, SpeedTestResult>("SELECT * FROM results WHERE id = $1")
                .bind(id)
                .fetch_optional(pool)
                .await
                .ok()
                .flatten()
        },
    };

    match result {
        Some(r) => {
            let response = ApiResponse {
                data: Some(ResultResponse::from(r)),
                message: "Success".to_string(),
            };
            (StatusCode::OK, Json(response))
        },
        None => {
            let response: ApiResponse<ResultResponse> = ApiResponse {
                data: None,
                message: "Result not found.".to_string(),
            };
            (StatusCode::NOT_FOUND, Json(response))
        }
    }
}

#[derive(Serialize)]
pub struct StatsResponse {
    pub total_results: i64,
    pub download: DownloadStats,
    pub upload: UploadStats,
    pub ping: PingStats,
}

#[derive(Serialize)]
pub struct DownloadStats {
    pub avg: i64,
    pub avg_bits: i64,
    pub avg_bits_human: String,
    pub max: i64,
    pub max_bits: i64,
    pub max_bits_human: String,
    pub min: i64,
    pub min_bits: i64,
    pub min_bits_human: String,
}

#[derive(Serialize)]
pub struct UploadStats {
    pub avg: i64,
    pub avg_bits: i64,
    pub avg_bits_human: String,
    pub max: i64,
    pub max_bits: i64,
    pub max_bits_human: String,
    pub min: i64,
    pub min_bits: i64,
    pub min_bits_human: String,
}

#[derive(Serialize)]
pub struct PingStats {
    pub avg: f64,
    pub max: f64,
    pub min: f64,
}

#[derive(sqlx::FromRow, Debug)]
struct StatsRow {
    total_results: i64,
    avg_ping: Option<f64>,
    avg_download: Option<f64>,
    avg_upload: Option<f64>,
    min_ping: Option<f64>,
    min_download: Option<f64>,
    min_upload: Option<f64>,
    max_ping: Option<f64>,
    max_download: Option<f64>,
    max_upload: Option<f64>,
}

// GET /api/v1/stats
pub async fn get_stats(State(state): State<AppState>) -> Json<ApiResponse<StatsResponse>> {
    let query = match &state.db {
        Database::Sqlite(pool) => {
            sqlx::query_as::<_, StatsRow>(
                "SELECT 
                    COUNT(*) as total_results,
                    AVG(ping) as avg_ping,
                    AVG(download) as avg_download,
                    AVG(upload) as avg_upload,
                    CAST(MIN(ping) AS REAL) as min_ping,
                    CAST(MIN(download) AS REAL) as min_download,
                    CAST(MIN(upload) AS REAL) as min_upload,
                    CAST(MAX(ping) AS REAL) as max_ping,
                    CAST(MAX(download) AS REAL) as max_download,
                    CAST(MAX(upload) AS REAL) as max_upload
                FROM results"
            )
            .fetch_one(pool)
            .await
        },
        Database::MySql(pool) => {
            sqlx::query_as::<_, StatsRow>(
                "SELECT 
                    COUNT(*) as total_results,
                    AVG(ping) as avg_ping,
                    AVG(download) as avg_download,
                    AVG(upload) as avg_upload,
                    CAST(MIN(ping) AS DECIMAL(10,2)) as min_ping,
                    CAST(MIN(download) AS DECIMAL(20,2)) as min_download,
                    CAST(MIN(upload) AS DECIMAL(20,2)) as min_upload,
                    CAST(MAX(ping) AS DECIMAL(10,2)) as max_ping,
                    CAST(MAX(download) AS DECIMAL(20,2)) as max_download,
                    CAST(MAX(upload) AS DECIMAL(20,2)) as max_upload
                FROM results"
            )
            .fetch_one(pool)
            .await
        },
        Database::Postgres(pool) => {
            sqlx::query_as::<_, StatsRow>(
                "SELECT 
                    COUNT(*) as total_results,
                    AVG(ping) as avg_ping,
                    AVG(download) as avg_download,
                    AVG(upload) as avg_upload,
                    CAST(MIN(ping) AS DOUBLE PRECISION) as min_ping,
                    CAST(MIN(download) AS DOUBLE PRECISION) as min_download,
                    CAST(MIN(upload) AS DOUBLE PRECISION) as min_upload,
                    CAST(MAX(ping) AS DOUBLE PRECISION) as max_ping,
                    CAST(MAX(download) AS DOUBLE PRECISION) as max_download,
                    CAST(MAX(upload) AS DOUBLE PRECISION) as max_upload
                FROM results"
            )
            .fetch_one(pool)
            .await
        },
    };
    let row = match query {
        Ok(r) => {
            tracing::debug!("Stats query successful: total={}, avg_download={:?}", r.total_results, r.avg_download);
            r
        },
        Err(e) => {
            tracing::error!("Stats query failed: {}", e);
            StatsRow {
                total_results: 0,
                avg_ping: None,
                avg_download: None,
                avg_upload: None,
                min_ping: None,
                min_download: None,
                min_upload: None,
                max_ping: None,
                max_download: None,
                max_upload: None,
            }
        }
    };
    
    let avg_download = row.avg_download.unwrap_or(0.0).round() as i64;
    let min_download = row.min_download.unwrap_or(0.0).round() as i64;
    let max_download = row.max_download.unwrap_or(0.0).round() as i64;
    let avg_upload = row.avg_upload.unwrap_or(0.0).round() as i64;
    let min_upload = row.min_upload.unwrap_or(0.0).round() as i64;
    let max_upload = row.max_upload.unwrap_or(0.0).round() as i64;
            
    let stats = StatsResponse {
        total_results: row.total_results,
        download: DownloadStats {
            avg: avg_download,
            avg_bits: avg_download * 8,
            avg_bits_human: format_bits((avg_download * 8) as f64),
            max: max_download,
            max_bits: max_download * 8,
            max_bits_human: format_bits((max_download * 8) as f64),
            min: min_download,
            min_bits: min_download * 8,
            min_bits_human: format_bits((min_download * 8) as f64),
        },
        upload: UploadStats {
            avg: avg_upload,
            avg_bits: avg_upload * 8,
            avg_bits_human: format_bits((avg_upload * 8) as f64),
            max: max_upload,
            max_bits: max_upload * 8,
            max_bits_human: format_bits((max_upload * 8) as f64),
            min: min_upload,
            min_bits: min_upload * 8,
            min_bits_human: format_bits((min_upload * 8) as f64),
        },
        ping: PingStats {
            avg: (row.avg_ping.unwrap_or(0.0) * 100.0).round() / 100.0,
            max: (row.max_ping.unwrap_or(0.0) * 100.0).round() / 100.0,
            min: (row.min_ping.unwrap_or(0.0) * 100.0).round() / 100.0,
        },
    };

    Json(ApiResponse {
        data: Some(stats),
        message: "ok".to_string(),
    })
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OoklaServer {
    pub id: String,
    pub host: String,
    pub name: String,
    pub location: String,
    pub country: String,
}

#[derive(Deserialize)]
struct OoklaApiServer {
    id: String,
    host: Option<String>,
    sponsor: Option<String>,
    name: Option<String>,
    country: Option<String>,
}

// GET /api/v1/ookla/list-servers
pub async fn list_ookla_servers() -> impl IntoResponse {
    match fetch_ookla_servers().await {
        Ok(servers) => {
            let response = ApiResponse {
                data: Some(servers),
                message: "Speedtest servers fetched successfully.".to_string(),
            };
            (StatusCode::OK, Json(response))
        },
        Err(e) => {
            tracing::error!("Unable to retrieve Ookla servers: {}", e);
            let response: ApiResponse<Vec<OoklaServer>> = ApiResponse {
                data: Some(vec![]),
                message: "Unable to retrieve Ookla servers, check internet connection and see logs.".to_string(),
            };
            (StatusCode::OK, Json(response))
        }
    }
}

async fn fetch_ookla_servers() -> Result<Vec<OoklaServer>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    
    let response = client
        .get("https://www.speedtest.net/api/js/servers")
        .query(&[
            ("engine", "js"),
            ("https_functional", "true"),
            ("limit", "20"),
        ])
        .send()
        .await?;
    
    let servers: Vec<OoklaApiServer> = response.json().await?;
    
    let mapped_servers = servers.into_iter()
        .map(|s| OoklaServer {
            id: s.id,
            host: s.host.unwrap_or_else(|| "Unknown".to_string()),
            name: s.sponsor.unwrap_or_else(|| "Unknown".to_string()),
            location: s.name.unwrap_or_else(|| "Unknown".to_string()),
            country: s.country.unwrap_or_else(|| "Unknown".to_string()),
        })
        .collect();
    
    Ok(mapped_servers)
}
