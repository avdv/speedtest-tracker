use askama_axum::Template;
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
    Form,
};
use crate::{models::{Result as SpeedTestResult, PersonalAccessToken}, db::Database, AppState};
use serde::Deserialize;

// Custom deserializer for checkbox arrays from HTML forms
fn deserialize_abilities<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    
    struct AbilitiesVisitor;
    
    impl<'de> Visitor<'de> for AbilitiesVisitor {
        type Value = Vec<String>;
        
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string or sequence of strings")
        }
        
        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(vec![value.to_string()])
        }
        
        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let mut vec = Vec::new();
            while let Some(value) = seq.next_element()? {
                vec.push(value);
            }
            Ok(vec)
        }
    }
    
    deserializer.deserialize_any(AbilitiesVisitor)
}

#[derive(Template)]
#[template(path = "results.html")]
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

fn default_page() -> i64 { 1 }
fn default_per_page() -> i64 { 25 }

pub async fn results_list(
    State(state): State<AppState>,
    Query(params): Query<Pagination>,
) -> ResultsListTemplate {
    let offset = (params.page - 1) * params.per_page;
    
    let results = match &state.db {
        Database::Sqlite(pool) => {
            sqlx::query_as::<_, SpeedTestResult>(
                "SELECT * FROM results ORDER BY created_at DESC LIMIT ? OFFSET ?"
            )
            .bind(params.per_page)
            .bind(offset)
            .fetch_all(pool)
            .await
            .unwrap_or_default()
        },
        Database::MySql(pool) => {
            sqlx::query_as::<_, SpeedTestResult>(
                "SELECT * FROM results ORDER BY created_at DESC LIMIT ? OFFSET ?"
            )
            .bind(params.per_page)
            .bind(offset)
            .fetch_all(pool)
            .await
            .unwrap_or_default()
        },
        Database::Postgres(pool) => {
            sqlx::query_as::<_, SpeedTestResult>(
                "SELECT * FROM results ORDER BY created_at DESC LIMIT $1 OFFSET $2"
            )
            .bind(params.per_page)
            .bind(offset)
            .fetch_all(pool)
            .await
            .unwrap_or_default()
        },
    };

    ResultsListTemplate {
        results,
        page: params.page,
        per_page: params.per_page,
    }
}

#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct HomeDashboardTemplate {
    latest_results: Vec<SpeedTestResult>,
    stats: DashboardStats,
    time_range: String,
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

fn default_time_range() -> String { "24h".to_string() }

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
    let time_cutoff = (chrono::Local::now() - chrono::Duration::hours(hours_ago))
        .naive_local();
    
    let (latest_results, stats) = match &state.db {
        Database::Sqlite(pool) => {
            let latest = sqlx::query_as::<_, SpeedTestResult>(
                "SELECT * FROM results ORDER BY created_at DESC LIMIT 5"
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
                "SELECT COUNT(*) FROM results WHERE created_at >= ? AND status = ?"
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
            
            let chart_data = chart_results.iter().map(|r| ChartDataPoint {
                timestamp: r.created_at.format("%Y-%m-%d %H:%M").to_string(),
                download: r.download_mbps(),
                upload: r.upload_mbps(),
                ping: r.ping.unwrap_or(0.0),
            }).collect();
            
            let stats = DashboardStats {
                total_tests: total,
                avg_download: avg_download.unwrap_or(0.0) * 8.0 / 1_000_000.0,
                avg_upload: avg_upload.unwrap_or(0.0) * 8.0 / 1_000_000.0,
                avg_ping: avg_ping.unwrap_or(0.0),
                chart_data,
            };
            
            (latest, stats)
        },
        Database::MySql(pool) => {
            let latest = sqlx::query_as::<_, SpeedTestResult>(
                "SELECT * FROM results ORDER BY created_at DESC LIMIT 5"
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
                "SELECT COUNT(*) FROM results WHERE created_at >= ? AND status = ?"
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
            
            let chart_data = chart_results.iter().map(|r| ChartDataPoint {
                timestamp: r.created_at.format("%Y-%m-%d %H:%M").to_string(),
                download: r.download_mbps(),
                upload: r.upload_mbps(),
                ping: r.ping.unwrap_or(0.0),
            }).collect();
            
            let stats = DashboardStats {
                total_tests: total,
                avg_download: avg_download.unwrap_or(0.0) * 8.0 / 1_000_000.0,
                avg_upload: avg_upload.unwrap_or(0.0) * 8.0 / 1_000_000.0,
                avg_ping: avg_ping.unwrap_or(0.0),
                chart_data,
            };
            
            (latest, stats)
        },
        Database::Postgres(pool) => {
            let latest = sqlx::query_as::<_, SpeedTestResult>(
                "SELECT * FROM results ORDER BY created_at DESC LIMIT 5"
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
                "SELECT COUNT(*) FROM results WHERE created_at >= $1 AND status = $2"
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
            
            let chart_data = chart_results.iter().map(|r| ChartDataPoint {
                timestamp: r.created_at.format("%Y-%m-%d %H:%M").to_string(),
                download: r.download_mbps(),
                upload: r.upload_mbps(),
                ping: r.ping.unwrap_or(0.0),
            }).collect();
            
            let stats = DashboardStats {
                total_tests: total,
                avg_download: avg_download.unwrap_or(0.0) * 8.0 / 1_000_000.0,
                avg_upload: avg_upload.unwrap_or(0.0) * 8.0 / 1_000_000.0,
                avg_ping: avg_ping.unwrap_or(0.0),
                chart_data,
            };
            
            (latest, stats)
        },
    };

    HomeDashboardTemplate {
        latest_results,
        stats,
        time_range: params.range.clone(),
    }
}

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    error: Option<String>,
}

pub async fn login_page() -> LoginTemplate {
    LoginTemplate { error: None }
}

#[derive(Deserialize)]
pub struct LoginForm {
    email: String,
    password: String,
}

pub async fn login_post(
    State(state): State<AppState>,
    Form(form): Form<LoginForm>,
) -> Response {
    let user = match &state.db {
        Database::Sqlite(pool) => {
            sqlx::query_as::<_, crate::models::User>(
                "SELECT * FROM users WHERE email = ?"
            )
            .bind(&form.email)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()
        },
        Database::MySql(pool) => {
            sqlx::query_as::<_, crate::models::User>(
                "SELECT * FROM users WHERE email = ?"
            )
            .bind(&form.email)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()
        },
        Database::Postgres(pool) => {
            sqlx::query_as::<_, crate::models::User>(
                "SELECT * FROM users WHERE email = $1"
            )
            .bind(&form.email)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()
        },
    };

    if let Some(user) = user {
        if bcrypt::verify(&form.password, &user.password).unwrap_or(false) {
            return Redirect::to("/").into_response();
        }
    }

    LoginTemplate {
        error: Some("Invalid credentials".to_string()),
    }.into_response()
}

#[derive(Template)]
#[template(path = "profile.html")]
pub struct ProfileTemplate {
    user: crate::models::User,
    message: Option<String>,
}

pub async fn profile_page(
    State(state): State<AppState>,
) -> ProfileTemplate {
    // TODO: Get actual logged-in user from session
    // For now, get first admin user as placeholder
    let user = match &state.db {
        Database::Sqlite(pool) => {
            sqlx::query_as::<_, crate::models::User>("SELECT * FROM users WHERE role = 'admin' LIMIT 1")
                .fetch_one(pool)
                .await
                .unwrap()
        },
        Database::MySql(pool) => {
            sqlx::query_as::<_, crate::models::User>("SELECT * FROM users WHERE role = 'admin' LIMIT 1")
                .fetch_one(pool)
                .await
                .unwrap()
        },
        Database::Postgres(pool) => {
            sqlx::query_as::<_, crate::models::User>("SELECT * FROM users WHERE role = 'admin' LIMIT 1")
                .fetch_one(pool)
                .await
                .unwrap()
        },
    };

    ProfileTemplate {
        user,
        message: None,
    }
}

#[derive(Deserialize)]
pub struct ProfileForm {
    name: String,
    email: String,
    password: Option<String>,
}

pub async fn profile_update(
    State(state): State<AppState>,
    Form(form): Form<ProfileForm>,
) -> Response {
    // TODO: Get actual user ID from session
    // For now, update first admin user
    
    let result = match &state.db {
        Database::Sqlite(pool) => {
            if let Some(password) = form.password {
                if !password.is_empty() {
                    let hashed = bcrypt::hash(&password, 12).unwrap();
                    sqlx::query(
                        "UPDATE users SET name = ?, email = ?, password = ?, updated_at = datetime('now') 
                         WHERE role = 'admin' LIMIT 1"
                    )
                    .bind(&form.name)
                    .bind(&form.email)
                    .bind(hashed)
                    .execute(pool)
                    .await
                } else {
                    sqlx::query(
                        "UPDATE users SET name = ?, email = ?, updated_at = datetime('now') 
                         WHERE role = 'admin' LIMIT 1"
                    )
                    .bind(&form.name)
                    .bind(&form.email)
                    .execute(pool)
                    .await
                }
            } else {
                sqlx::query(
                    "UPDATE users SET name = ?, email = ?, updated_at = datetime('now') 
                     WHERE role = 'admin' LIMIT 1"
                )
                .bind(&form.name)
                .bind(&form.email)
                .execute(pool)
                .await
            }
        },
        _ => return Redirect::to("/admin/profile").into_response(),
    };

    if result.is_ok() {
        Redirect::to("/admin/profile?updated=1").into_response()
    } else {
        Redirect::to("/admin/profile?error=1").into_response()
    }
}

#[derive(Template)]
#[template(path = "api-tokens.html")]
pub struct ApiTokensTemplate {
    tokens: Vec<PersonalAccessToken>,
    message: Option<String>,
    new_token: Option<String>,
    new_token_name: Option<String>,
}

pub async fn api_tokens_page(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> ApiTokensTemplate {
    let message = if params.contains_key("deleted") {
        Some("Token deleted successfully!".to_string())
    } else if params.contains_key("error") {
        Some("An error occurred.".to_string())
    } else {
        None
    };

    let new_token = params.get("token").map(|s| s.to_string());
    let new_token_name = params.get("token_name").map(|s| s.to_string());

    let tokens = match &state.db {
        Database::Sqlite(pool) => {
            sqlx::query_as::<_, PersonalAccessToken>(
                "SELECT * FROM personal_access_tokens ORDER BY created_at DESC"
            )
            .fetch_all(pool)
            .await
            .unwrap_or_default()
        },
        Database::MySql(pool) => {
            sqlx::query_as::<_, PersonalAccessToken>(
                "SELECT * FROM personal_access_tokens ORDER BY created_at DESC"
            )
            .fetch_all(pool)
            .await
            .unwrap_or_default()
        },
        Database::Postgres(pool) => {
            sqlx::query_as::<_, PersonalAccessToken>(
                "SELECT * FROM personal_access_tokens ORDER BY created_at DESC"
            )
            .fetch_all(pool)
            .await
            .unwrap_or_default()
        },
    };

    ApiTokensTemplate {
        tokens,
        message,
        new_token,
        new_token_name,
    }
}

#[derive(Deserialize)]
pub struct CreateTokenForm {
    name: String,
    #[serde(default, deserialize_with = "deserialize_abilities")]
    abilities: Vec<String>,
}

pub async fn create_token(
    State(state): State<AppState>,
    Form(form): Form<CreateTokenForm>,
) -> Response {
    use rand::Rng;
    use sha2::{Sha256, Digest};
    
    // Generate random token (plaintext) - 40 characters like Laravel Sanctum
    let token: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(40)
        .map(char::from)
        .collect();
    
    // Hash the token using SHA-256 (same as Laravel Sanctum)
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let hash_bytes = hasher.finalize();
    let hashed = hex::encode(hash_bytes);
    
    // Convert abilities to JSON
    let abilities_json = serde_json::to_string(&form.abilities).unwrap_or_else(|_| "[]".to_string());
    
    let result = match &state.db {
        Database::Sqlite(pool) => {
            // Get first admin user ID
            let user_id: i64 = sqlx::query_scalar("SELECT id FROM users WHERE role = 'admin' LIMIT 1")
                .fetch_one(pool)
                .await
                .unwrap_or(1);
            
            sqlx::query(
                "INSERT INTO personal_access_tokens 
                 (tokenable_type, tokenable_id, name, token, abilities, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, datetime('now'), datetime('now'))"
            )
            .bind("App\\Models\\User")
            .bind(user_id)
            .bind(&form.name)
            .bind(&hashed)
            .bind(&abilities_json)
            .execute(pool)
            .await
        },
        _ => return Redirect::to("/admin/api-tokens").into_response(),
    };

    if result.is_ok() {
        // Redirect with token in query string to display it (only once!)
        let redirect_url = format!(
            "/admin/api-tokens?token={}&token_name={}",
            urlencoding::encode(&token),
            urlencoding::encode(&form.name)
        );
        Redirect::to(&redirect_url).into_response()
    } else {
        Redirect::to("/admin/api-tokens?error=1").into_response()
    }
}

#[derive(Deserialize)]
pub struct DeleteTokenForm {
    id: i64,
}

pub async fn delete_token(
    State(state): State<AppState>,
    Form(form): Form<DeleteTokenForm>,
) -> Response {
    let success = match &state.db {
        Database::Sqlite(pool) => {
            sqlx::query("DELETE FROM personal_access_tokens WHERE id = ?")
                .bind(form.id)
                .execute(pool)
                .await
                .is_ok()
        },
        Database::MySql(pool) => {
            sqlx::query("DELETE FROM personal_access_tokens WHERE id = ?")
                .bind(form.id)
                .execute(pool)
                .await
                .is_ok()
        },
        Database::Postgres(pool) => {
            sqlx::query("DELETE FROM personal_access_tokens WHERE id = $1")
                .bind(form.id)
                .execute(pool)
                .await
                .is_ok()
        },
    };

    if success {
        Redirect::to("/admin/api-tokens?deleted=1").into_response()
    } else {
        Redirect::to("/admin/api-tokens?error=1").into_response()
    }
}
