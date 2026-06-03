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
    pub comments: Option<String>,
    pub data: Option<String>,
    pub status: String,
    pub scheduled: bool,
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

impl Result {
    pub fn download_mbps(&self) -> f64 {
        // Database stores bytes/second, multiply by 8 to get bits/second, then divide by 1M for Mbps
        self.download.unwrap_or(0) as f64 * 8.0 / 1_000_000.0
    }

    pub fn upload_mbps(&self) -> f64 {
        // Database stores bytes/second, multiply by 8 to get bits/second, then divide by 1M for Mbps
        self.upload.unwrap_or(0) as f64 * 8.0 / 1_000_000.0
    }
}
