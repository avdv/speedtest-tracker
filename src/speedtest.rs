use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct SpeedtestResult {
    pub download: i64,       // bytes per second (bandwidth)
    pub upload: i64,         // bytes per second (bandwidth)
    pub download_bytes: i64, // total bytes transferred
    pub upload_bytes: i64,   // total bytes transferred
    pub ping: f64,           // milliseconds
    pub server_id: Option<i64>,
    pub server_name: Option<String>,
    pub server_location: Option<String>,
    pub server_country: Option<String>,
    pub data: String, // Full JSON response
}

#[derive(Debug, Serialize, Deserialize)]
struct OoklaResult {
    #[serde(rename = "type")]
    result_type: String,
    timestamp: String,
    ping: OoklaPing,
    download: OoklaSpeed,
    upload: OoklaSpeed,
    #[serde(rename = "packetLoss")]
    packet_loss: Option<f64>,
    isp: String,
    interface: OoklaInterface,
    server: OoklaServer,
    result: OoklaResultInfo,
}

#[derive(Debug, Serialize, Deserialize)]
struct OoklaPing {
    jitter: f64,
    latency: f64,
    low: Option<f64>,
    high: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OoklaSpeed {
    bandwidth: i64, // bytes per second
    bytes: i64,
    elapsed: i64,
    latency: Option<OoklaLatency>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OoklaLatency {
    iqm: f64,
    low: f64,
    high: f64,
    jitter: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct OoklaInterface {
    #[serde(rename = "internalIp")]
    internal_ip: String,
    name: String,
    #[serde(rename = "macAddr")]
    mac_addr: String,
    #[serde(rename = "isVpn")]
    is_vpn: bool,
    #[serde(rename = "externalIp")]
    external_ip: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OoklaServer {
    id: i64,
    host: String,
    port: i64,
    name: String,
    location: String,
    country: String,
    ip: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OoklaResultInfo {
    id: String,
    url: String,
    persisted: bool,
}

pub async fn run_speedtest(server_id: Option<i64>) -> Result<SpeedtestResult, String> {
    tracing::info!(
        "Starting speedtest{}",
        if let Some(id) = server_id {
            format!(" with server {}", id)
        } else {
            String::new()
        }
    );

    // Build command
    let mut cmd = Command::new("speedtest");
    cmd.arg("--accept-license")
        .arg("--accept-gdpr")
        .arg("--format=json");

    if let Some(id) = server_id {
        cmd.arg(format!("--server-id={}", id));
    }

    tracing::debug!("Executing: {:?}", cmd);

    // Run speedtest
    let output = cmd.output().map_err(|e| {
        format!(
            "Failed to execute speedtest command: {}. Is 'speedtest' CLI installed?",
            e
        )
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::error!("Speedtest failed: {}", stderr);
        return Err(format!("Speedtest failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    tracing::debug!("Speedtest raw output: {}", stdout);

    // Parse JSON result
    let ookla_result: OoklaResult = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse speedtest result: {}", e))?;

    // Convert to our format
    let result = SpeedtestResult {
        download: ookla_result.download.bandwidth,
        upload: ookla_result.upload.bandwidth,
        download_bytes: ookla_result.download.bytes,
        upload_bytes: ookla_result.upload.bytes,
        ping: ookla_result.ping.latency,
        server_id: Some(ookla_result.server.id),
        server_name: Some(ookla_result.server.name.clone()),
        server_location: Some(ookla_result.server.location.clone()),
        server_country: Some(ookla_result.server.country.clone()),
        data: stdout.to_string(),
    };

    tracing::info!(
        "Speedtest completed: {:.2} Mbps down, {:.2} Mbps up, {:.1} ms ping",
        result.download as f64 * 8.0 / 1_000_000.0,
        result.upload as f64 * 8.0 / 1_000_000.0,
        result.ping
    );

    Ok(result)
}

pub async fn save_result(
    db: &crate::db::Database,
    result: SpeedtestResult,
    scheduled: bool,
) -> Result<i64, String> {
    use crate::db::Database;

    let result_id = match db {
        #[cfg(feature = "sqlite")]

        Database::Sqlite(pool) => {
            sqlx::query(
                "INSERT INTO results (service, ping, download, upload, download_bytes, upload_bytes, data, status, scheduled, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))"
            )
            .bind("ookla")
            .bind(result.ping)
            .bind(result.download)
            .bind(result.upload)
            .bind(result.download_bytes)
            .bind(result.upload_bytes)
            .bind(&result.data)
            .bind("completed")
            .bind(scheduled)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to save result: {}", e))?
            .last_insert_rowid()
        },
        #[cfg(feature = "mysql")]

        Database::MySql(pool) => {
            sqlx::query(
                "INSERT INTO results (service, ping, download, upload, download_bytes, upload_bytes, data, status, scheduled, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, NOW(), NOW())"
            )
            .bind("ookla")
            .bind(result.ping)
            .bind(result.download)
            .bind(result.upload)
            .bind(result.download_bytes)
            .bind(result.upload_bytes)
            .bind(&result.data)
            .bind("completed")
            .bind(scheduled)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to save result: {}", e))?
            .last_insert_id() as i64
        },
        #[cfg(feature = "postgres")]

        Database::Postgres(pool) => {
            sqlx::query_scalar(
                "INSERT INTO results (service, ping, download, upload, download_bytes, upload_bytes, data, status, scheduled, created_at, updated_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())
                 RETURNING id"
            )
            .bind("ookla")
            .bind(result.ping)
            .bind(result.download)
            .bind(result.upload)
            .bind(result.download_bytes)
            .bind(result.upload_bytes)
            .bind(&result.data)
            .bind("completed")
            .bind(scheduled)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Failed to save result: {}", e))?
        },
    };

    tracing::info!("Saved speedtest result with ID: {}", result_id);
    Ok(result_id)
}
