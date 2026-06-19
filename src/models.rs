use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Result {
    pub id: i64,
    pub service: String,
    pub ping: Option<f64>,
    pub download: Option<i64>,
    pub upload: Option<i64>,
    pub download_bytes: Option<i64>,
    pub upload_bytes: Option<i64>,
    pub comments: Option<String>,
    pub data: Option<String>,
    pub status: String,
    pub scheduled: bool,
    pub healthy: Option<bool>,
    pub benchmarks: Option<String>,
    pub dispatched_by: Option<i64>,
    #[serde(with = "naive_datetime_as_utc")]
    pub created_at: NaiveDateTime,
    #[serde(with = "naive_datetime_as_utc")]
    pub updated_at: NaiveDateTime,
}

mod naive_datetime_as_utc {
    use chrono::NaiveDateTime;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(date: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&date.format("%Y-%m-%d %H:%M:%S").to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S").map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PersonalAccessToken {
    pub id: i64,
    pub tokenable_type: String,
    pub tokenable_id: i64,
    pub name: String,
    pub token: String,
    pub abilities: Option<String>,
    pub last_used_at: Option<NaiveDateTime>,
    pub expires_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl PersonalAccessToken {
    #[must_use]
    pub fn is_read(&self) -> bool {
        if let Some(abilities) = &self.abilities {
            return abilities.contains("read");
        }
        false
    }
}

impl Result {
    #[must_use]
    pub fn download_mbps(&self) -> f64 {
        // Database stores bandwidth in bytes/second, multiply by 8 to get bits/second, then divide by 1M for Mbps
        self.download.unwrap_or(0) as f64 * 8.0 / 1_000_000.0
    }

    #[must_use]
    pub fn upload_mbps(&self) -> f64 {
        // Database stores bandwidth in bytes/second, multiply by 8 to get bits/second, then divide by 1M for Mbps
        self.upload.unwrap_or(0) as f64 * 8.0 / 1_000_000.0
    }

    #[must_use]
    pub fn server_info(&self) -> Option<ServerInfo> {
        // Parse server info from JSON data field if available
        self.data.as_ref().and_then(|json_str| {
            serde_json::from_str::<serde_json::Value>(json_str)
                .ok()
                .and_then(|data| {
                    let server_name = data.get("server")?.get("name")?.as_str().map(String::from);
                    let server_host = data.get("server")?.get("host")?.as_str().map(String::from);
                    let server_location = data
                        .get("server")?
                        .get("location")?
                        .as_str()
                        .map(String::from);
                    let server_country = data
                        .get("server")?
                        .get("country")?
                        .as_str()
                        .map(String::from);

                    Some(ServerInfo {
                        name: server_name,
                        host: server_host,
                        location: server_location,
                        country: server_country,
                    })
                })
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: Option<String>,
    pub host: Option<String>,
    pub location: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Schedule {
    pub id: i64,
    pub name: String,
    pub cron: String,
    pub server_ids: Option<String>,
    pub enabled: bool,
    pub last_run_at: Option<NaiveDateTime>,
    pub next_run_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Schedule {
    #[allow(dead_code)]
    #[must_use]
    pub fn get_server_ids(&self) -> Vec<i64> {
        self.server_ids
            .as_ref()
            .map(|s| {
                s.split(',')
                    .filter_map(|id| id.trim().parse::<i64>().ok())
                    .collect()
            })
            .unwrap_or_default()
    }
}
