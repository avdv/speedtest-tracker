use askama_axum::Template;
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
    Form,
};
use crate::{models::{Result as SpeedTestResult, PersonalAccessToken}, db::Database, AppState};
use serde::Deserialize;

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
    session: tower_sessions::Session,
    Form(form): Form<LoginForm>,
) -> Response {
    tracing::debug!("Login attempt for email: {}", form.email);
    
    let user = match &state.db {
        Database::Sqlite(pool) => {
            sqlx::query_as::<_, crate::models::User>(
                "SELECT * FROM users WHERE email = ?"
            )
            .bind(&form.email)
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                tracing::error!("Database query error during login: {}", e);
                e
            })
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
            .map_err(|e| {
                tracing::error!("Database query error during login: {}", e);
                e
            })
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
            .map_err(|e| {
                tracing::error!("Database query error during login: {}", e);
                e
            })
            .ok()
            .flatten()
        },
    };

    if let Some(user) = user {
        tracing::debug!("User found, verifying password");
        match bcrypt::verify(&form.password, &user.password) {
            Ok(true) => {
                tracing::debug!("Password verified, creating session");
                // Set session
                if let Err(e) = crate::session::set_user_session(session, user.id).await {
                    tracing::error!("Failed to set session: {}", e);
                    return LoginTemplate {
                        error: Some(format!("Login failed - session error: {}", e)),
                    }.into_response();
                }
                tracing::info!("User {} logged in successfully, attempting redirect", user.email);
                
                // Try different redirect approaches
                let response = Redirect::to("/");
                tracing::debug!("Redirect response created");
                return response.into_response();
            }
            Ok(false) => {
                tracing::debug!("Password verification failed");
            }
            Err(e) => {
                tracing::error!("Password verification error: {}", e);
            }
        }
    } else {
        tracing::debug!("User not found");
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
    session: tower_sessions::Session,
) -> Response {
    // Get logged-in user from session
    let user_id = match crate::session::get_user_id(session).await {
        Some(id) => id,
        None => return Redirect::to("/login").into_response(),
    };
    
    let user = match &state.db {
        Database::Sqlite(pool) => {
            sqlx::query_as::<_, crate::models::User>("SELECT * FROM users WHERE id = ?")
                .bind(user_id)
                .fetch_optional(pool)
                .await
        },
        Database::MySql(pool) => {
            sqlx::query_as::<_, crate::models::User>("SELECT * FROM users WHERE id = ?")
                .bind(user_id)
                .fetch_optional(pool)
                .await
        },
        Database::Postgres(pool) => {
            sqlx::query_as::<_, crate::models::User>("SELECT * FROM users WHERE id = $1")
                .bind(user_id)
                .fetch_optional(pool)
                .await
        },
    };

    match user {
        Ok(Some(user)) => ProfileTemplate {
            user,
            message: None,
        }.into_response(),
        _ => Redirect::to("/login").into_response(),
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

pub async fn create_token(
    State(state): State<AppState>,
    body: String,
) -> Response {
    use rand::Rng;
    use sha2::{Sha256, Digest};
    
    // Parse form manually to handle duplicate keys
    let mut name = String::new();
    let mut abilities: Vec<String> = Vec::new();
    
    for pair in body.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            let decoded_value = urlencoding::decode(value).unwrap_or_default();
            match key {
                "name" => name = decoded_value.to_string(),
                "abilities" => abilities.push(decoded_value.to_string()),
                _ => {}
            }
        }
    }
    
    // Default to results:read if no abilities selected
    if abilities.is_empty() {
        abilities.push("results:read".to_string());
    }
    
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
    let abilities_json = serde_json::to_string(&abilities).unwrap_or_else(|_| "[]".to_string());
    
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
            .bind(&name)
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
            urlencoding::encode(&name)
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

pub async fn logout(
    session: tower_sessions::Session,
) -> Response {
    if let Err(e) = crate::session::clear_session(session).await {
        tracing::error!("Failed to clear session: {}", e);
    }
    Redirect::to("/login").into_response()
}
