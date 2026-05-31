use chrono::{DateTime, Utc};
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PersonalAccessToken {
    pub id: i64,
    pub tokenable_type: String,
    pub tokenable_id: i64,
    pub name: String,
    pub token: String,
    pub abilities: Option<String>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Result {
    pub fn download_mbps(&self) -> f64 {
        self.download.unwrap_or(0) as f64 / 1_000_000.0
    }

    pub fn upload_mbps(&self) -> f64 {
        self.upload.unwrap_or(0) as f64 / 1_000_000.0
    }
}
