use crate::{
    AppState,
    db::Database,
    models::{PersonalAccessToken, Result as SpeedTestResult},
};
use askama_axum::Template;
use axum::{
    Form,
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;

#[derive(Template)]
#[template(path = "pages/results.html")]
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

fn default_page() -> i64 {
    1
}
fn default_per_page() -> i64 {
    25
}

pub async fn results_list(
    State(state): State<AppState>,
    Query(params): Query<Pagination>,
) -> ResultsListTemplate {
    let offset = (params.page - 1) * params.per_page;

    let results = match &state.db {
        #[cfg(feature = "sqlite")]

        Database::Sqlite(pool) => sqlx::query_as::<_, SpeedTestResult>(
            "SELECT * FROM results ORDER BY created_at DESC LIMIT ? OFFSET ?",
        )
        .bind(params.per_page)
        .bind(offset)
        .fetch_all(pool)
        .await
        .unwrap_or_default(),
        #[cfg(feature = "mysql")]

        Database::MySql(pool) => sqlx::query_as::<_, SpeedTestResult>(
            "SELECT * FROM results ORDER BY created_at DESC LIMIT ? OFFSET ?",
        )
        .bind(params.per_page)
        .bind(offset)
        .fetch_all(pool)
        .await
        .unwrap_or_default(),
        #[cfg(feature = "postgres")]

        Database::Postgres(pool) => sqlx::query_as::<_, SpeedTestResult>(
            "SELECT * FROM results ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(params.per_page)
        .bind(offset)
        .fetch_all(pool)
        .await
        .unwrap_or_default(),
    };

    ResultsListTemplate {
        results,
        page: params.page,
        per_page: params.per_page,
    }
}

#[derive(Deserialize)]
pub struct DeleteResultsForm {
    ids: String,
}

pub async fn delete_results(
    State(state): State<AppState>,
    Form(form): Form<DeleteResultsForm>,
) -> impl IntoResponse {
    // Parse comma-separated IDs
    let ids: Vec<i64> = form
        .ids
        .split(',')
        .filter_map(|s| s.trim().parse::<i64>().ok())
        .collect();

    if ids.is_empty() {
        return Redirect::to("/admin/results");
    }

    // Delete results based on database type
    match &state.db {
        #[cfg(feature = "sqlite")]

        Database::Sqlite(pool) => {
            for id in ids {
                let _ = sqlx::query("DELETE FROM results WHERE id = ?")
                    .bind(id)
                    .execute(pool)
                    .await;
            }
        }
        #[cfg(feature = "mysql")]

        Database::MySql(pool) => {
            for id in ids {
                let _ = sqlx::query("DELETE FROM results WHERE id = ?")
                    .bind(id)
                    .execute(pool)
                    .await;
            }
        }
        #[cfg(feature = "postgres")]

        Database::Postgres(pool) => {
            for id in ids {
                let _ = sqlx::query("DELETE FROM results WHERE id = $1")
                    .bind(id)
                    .execute(pool)
                    .await;
            }
        }
    }

    Redirect::to("/admin/results")
}

#[derive(Template)]
#[template(path = "pages/dashboard.html")]
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
        #[cfg(feature = "sqlite")]

        Database::Sqlite(pool) => {
            sqlx::query_as::<_, crate::models::User>("SELECT * FROM users WHERE email = ?")
                .bind(&form.email)
                .fetch_optional(pool)
                .await
                .map_err(|e| {
                    tracing::error!("Database query error during login: {}", e);
                    e
                })
                .ok()
                .flatten()
        }
        #[cfg(feature = "mysql")]

        Database::MySql(pool) => {
            sqlx::query_as::<_, crate::models::User>("SELECT * FROM users WHERE email = ?")
                .bind(&form.email)
                .fetch_optional(pool)
                .await
                .map_err(|e| {
                    tracing::error!("Database query error during login: {}", e);
                    e
                })
                .ok()
                .flatten()
        }
        #[cfg(feature = "postgres")]

        Database::Postgres(pool) => {
            sqlx::query_as::<_, crate::models::User>("SELECT * FROM users WHERE email = $1")
                .bind(&form.email)
                .fetch_optional(pool)
                .await
                .map_err(|e| {
                    tracing::error!("Database query error during login: {}", e);
                    e
                })
                .ok()
                .flatten()
        }
    };

    if let Some(user) = user {
        tracing::debug!("User found, verifying password");
        match bcrypt::verify(&form.password, &user.password) {
            Ok(true) => {
                tracing::debug!("Password verified, creating session");

                // Check if there's a redirect URL stored
                let redirect_url = session
                    .get::<String>("redirect_after_login")
                    .await
                    .ok()
                    .flatten()
                    .unwrap_or_else(|| "/".to_string());

                // Clear the redirect URL from session
                let _ = session.remove::<String>("redirect_after_login").await;

                // Set session
                if let Err(e) = crate::session::set_user_session(session, user.id).await {
                    tracing::error!("Failed to set session: {}", e);
                    return LoginTemplate {
                        error: Some(format!("Login failed - session error: {}", e)),
                    }
                    .into_response();
                }

                tracing::info!(
                    "User {} logged in successfully, redirecting to {}",
                    user.email,
                    redirect_url
                );
                return Redirect::to(&redirect_url).into_response();
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
    }
    .into_response()
}

#[derive(Template)]
#[template(path = "pages/profile.html")]
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
        #[cfg(feature = "sqlite")]

        Database::Sqlite(pool) => {
            sqlx::query_as::<_, crate::models::User>("SELECT * FROM users WHERE id = ?")
                .bind(user_id)
                .fetch_optional(pool)
                .await
        }
        #[cfg(feature = "mysql")]

        Database::MySql(pool) => {
            sqlx::query_as::<_, crate::models::User>("SELECT * FROM users WHERE id = ?")
                .bind(user_id)
                .fetch_optional(pool)
                .await
        }
        #[cfg(feature = "postgres")]

        Database::Postgres(pool) => {
            sqlx::query_as::<_, crate::models::User>("SELECT * FROM users WHERE id = $1")
                .bind(user_id)
                .fetch_optional(pool)
                .await
        }
    };

    match user {
        Ok(Some(user)) => ProfileTemplate {
            user,
            message: None,
        }
        .into_response(),
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

    #[allow(unreachable_patterns)]
    let success = match &state.db {
        #[cfg(feature = "sqlite")]

        Database::Sqlite(pool) => {
            let result = if let Some(password) = form.password {
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
                         WHERE role = 'admin' LIMIT 1",
                    )
                    .bind(&form.name)
                    .bind(&form.email)
                    .execute(pool)
                    .await
                }
            } else {
                sqlx::query(
                    "UPDATE users SET name = ?, email = ?, updated_at = datetime('now') 
                     WHERE role = 'admin' LIMIT 1",
                )
                .bind(&form.name)
                .bind(&form.email)
                .execute(pool)
                .await
            };
            result.is_ok()
        }
        #[cfg(feature = "mysql")]

        Database::MySql(pool) => {
            let result = if let Some(password) = form.password {
                if !password.is_empty() {
                    let hashed = bcrypt::hash(&password, 12).unwrap();
                    sqlx::query(
                        "UPDATE users SET name = ?, email = ?, password = ?, updated_at = NOW() 
                         WHERE role = 'admin' LIMIT 1",
                    )
                    .bind(&form.name)
                    .bind(&form.email)
                    .bind(hashed)
                    .execute(pool)
                    .await
                } else {
                    sqlx::query(
                        "UPDATE users SET name = ?, email = ?, updated_at = NOW() 
                         WHERE role = 'admin' LIMIT 1",
                    )
                    .bind(&form.name)
                    .bind(&form.email)
                    .execute(pool)
                    .await
                }
            } else {
                sqlx::query(
                    "UPDATE users SET name = ?, email = ?, updated_at = NOW() 
                     WHERE role = 'admin' LIMIT 1",
                )
                .bind(&form.name)
                .bind(&form.email)
                .execute(pool)
                .await
            };
            result.is_ok()
        }
        #[cfg(feature = "postgres")]

        Database::Postgres(pool) => {
            let result = if let Some(password) = form.password {
                if !password.is_empty() {
                    let hashed = bcrypt::hash(&password, 12).unwrap();
                    sqlx::query(
                        "UPDATE users SET name = $1, email = $2, password = $3, updated_at = NOW() 
                         WHERE role = 'admin' LIMIT 1",
                    )
                    .bind(&form.name)
                    .bind(&form.email)
                    .bind(hashed)
                    .execute(pool)
                    .await
                } else {
                    sqlx::query(
                        "UPDATE users SET name = $1, email = $2, updated_at = NOW() 
                         WHERE role = 'admin' LIMIT 1",
                    )
                    .bind(&form.name)
                    .bind(&form.email)
                    .execute(pool)
                    .await
                }
            } else {
                sqlx::query(
                    "UPDATE users SET name = $1, email = $2, updated_at = NOW() 
                     WHERE role = 'admin' LIMIT 1",
                )
                .bind(&form.name)
                .bind(&form.email)
                .execute(pool)
                .await
            };
            result.is_ok()
        }
        _ => return Redirect::to("/admin/profile").into_response(),
    };

    if success {
        Redirect::to("/admin/profile?updated=1").into_response()
    } else {
        Redirect::to("/admin/profile?error=1").into_response()
    }
}

#[derive(Template)]
#[template(path = "pages/api-tokens.html")]
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
        #[cfg(feature = "sqlite")]

        Database::Sqlite(pool) => sqlx::query_as::<_, PersonalAccessToken>(
            "SELECT * FROM personal_access_tokens ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await
        .unwrap_or_default(),
        #[cfg(feature = "mysql")]

        Database::MySql(pool) => sqlx::query_as::<_, PersonalAccessToken>(
            "SELECT * FROM personal_access_tokens ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await
        .unwrap_or_default(),
        #[cfg(feature = "postgres")]

        Database::Postgres(pool) => sqlx::query_as::<_, PersonalAccessToken>(
            "SELECT * FROM personal_access_tokens ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await
        .unwrap_or_default(),
    };

    ApiTokensTemplate {
        tokens,
        message,
        new_token,
        new_token_name,
    }
}

pub async fn create_token(State(state): State<AppState>, body: String) -> Response {
    use rand::Rng;
    use sha2::{Digest, Sha256};

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

    #[allow(unreachable_patterns)]
    let success = match &state.db {
        #[cfg(feature = "sqlite")]

        Database::Sqlite(pool) => {
            // Get first admin user ID
            let user_id: i64 =
                sqlx::query_scalar("SELECT id FROM users WHERE role = 'admin' LIMIT 1")
                    .fetch_one(pool)
                    .await
                    .unwrap_or(1);

            let result = sqlx::query(
                "INSERT INTO personal_access_tokens 
                 (tokenable_type, tokenable_id, name, token, abilities, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, datetime('now'), datetime('now'))",
            )
            .bind("App\\Models\\User")
            .bind(user_id)
            .bind(&name)
            .bind(&hashed)
            .bind(&abilities_json)
            .execute(pool)
            .await;
            result.is_ok()
        }
        #[cfg(feature = "mysql")]

        Database::MySql(pool) => {
            // Get first admin user ID
            let user_id: i64 =
                sqlx::query_scalar("SELECT id FROM users WHERE role = 'admin' LIMIT 1")
                    .fetch_one(pool)
                    .await
                    .unwrap_or(1);

            let result = sqlx::query(
                "INSERT INTO personal_access_tokens 
                 (tokenable_type, tokenable_id, name, token, abilities, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, NOW(), NOW())",
            )
            .bind("App\\Models\\User")
            .bind(user_id)
            .bind(&name)
            .bind(&hashed)
            .bind(&abilities_json)
            .execute(pool)
            .await;
            result.is_ok()
        }
        #[cfg(feature = "postgres")]

        Database::Postgres(pool) => {
            // Get first admin user ID
            let user_id: i64 =
                sqlx::query_scalar("SELECT id FROM users WHERE role = 'admin' LIMIT 1")
                    .fetch_one(pool)
                    .await
                    .unwrap_or(1);

            let result = sqlx::query(
                "INSERT INTO personal_access_tokens 
                 (tokenable_type, tokenable_id, name, token, abilities, created_at, updated_at)
                 VALUES ($1, $2, $3, $4, $5, NOW(), NOW())",
            )
            .bind("App\\Models\\User")
            .bind(user_id)
            .bind(&name)
            .bind(&hashed)
            .bind(&abilities_json)
            .execute(pool)
            .await;
            result.is_ok()
        }
        _ => return Redirect::to("/admin/api-tokens").into_response(),
    };

    if success {
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
        #[cfg(feature = "sqlite")]

        Database::Sqlite(pool) => sqlx::query("DELETE FROM personal_access_tokens WHERE id = ?")
            .bind(form.id)
            .execute(pool)
            .await
            .is_ok(),
        #[cfg(feature = "mysql")]

        Database::MySql(pool) => sqlx::query("DELETE FROM personal_access_tokens WHERE id = ?")
            .bind(form.id)
            .execute(pool)
            .await
            .is_ok(),
        #[cfg(feature = "postgres")]

        Database::Postgres(pool) => sqlx::query("DELETE FROM personal_access_tokens WHERE id = $1")
            .bind(form.id)
            .execute(pool)
            .await
            .is_ok(),
    };

    if success {
        Redirect::to("/admin/api-tokens?deleted=1").into_response()
    } else {
        Redirect::to("/admin/api-tokens?error=1").into_response()
    }
}

pub async fn logout(session: tower_sessions::Session) -> Response {
    if let Err(e) = crate::session::clear_session(session).await {
        tracing::error!("Failed to clear session: {}", e);
    }
    Redirect::to("/login").into_response()
}

#[derive(Template)]
#[template(path = "pages/edit-token.html")]
pub struct EditTokenTemplate {
    token: PersonalAccessToken,
    error: Option<String>,
}

pub async fn edit_token_page(
    State(state): State<AppState>,
    axum::extract::Path(token_id): axum::extract::Path<i64>,
) -> Response {
    let token = match &state.db {
        #[cfg(feature = "sqlite")]

        Database::Sqlite(pool) => {
            sqlx::query_as::<_, PersonalAccessToken>(
                "SELECT * FROM personal_access_tokens WHERE id = ?",
            )
            .bind(token_id)
            .fetch_optional(pool)
            .await
        }
        #[cfg(feature = "mysql")]

        Database::MySql(pool) => {
            sqlx::query_as::<_, PersonalAccessToken>(
                "SELECT * FROM personal_access_tokens WHERE id = ?",
            )
            .bind(token_id)
            .fetch_optional(pool)
            .await
        }
        #[cfg(feature = "postgres")]

        Database::Postgres(pool) => {
            sqlx::query_as::<_, PersonalAccessToken>(
                "SELECT * FROM personal_access_tokens WHERE id = $1",
            )
            .bind(token_id)
            .fetch_optional(pool)
            .await
        }
    };

    match token {
        Ok(Some(token)) => EditTokenTemplate { token, error: None }.into_response(),
        _ => Redirect::to("/admin/api-tokens").into_response(),
    }
}

pub async fn update_token(State(state): State<AppState>, body: String) -> Response {
    let mut token_id: Option<i64> = None;
    let mut name = String::new();
    let mut abilities: Vec<String> = Vec::new();

    for pair in body.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            let decoded_value = urlencoding::decode(value).unwrap_or_default();
            match key {
                "id" => token_id = decoded_value.parse().ok(),
                "name" => name = decoded_value.to_string(),
                "abilities" => abilities.push(decoded_value.to_string()),
                _ => {}
            }
        }
    }

    if abilities.is_empty() {
        abilities.push("results:read".to_string());
    }

    let Some(id) = token_id else {
        return Redirect::to("/admin/api-tokens?error=1").into_response();
    };

    let abilities_json = serde_json::to_string(&abilities).unwrap_or_else(|_| "[]".to_string());

    let success = match &state.db {
        #[cfg(feature = "sqlite")]

        Database::Sqlite(pool) => sqlx::query(
            "UPDATE personal_access_tokens 
                 SET name = ?, abilities = ?, updated_at = datetime('now')
                 WHERE id = ?",
        )
        .bind(&name)
        .bind(&abilities_json)
        .bind(id)
        .execute(pool)
        .await
        .is_ok(),
        #[cfg(feature = "mysql")]

        Database::MySql(pool) => sqlx::query(
            "UPDATE personal_access_tokens 
                 SET name = ?, abilities = ?, updated_at = NOW()
                 WHERE id = ?",
        )
        .bind(&name)
        .bind(&abilities_json)
        .bind(id)
        .execute(pool)
        .await
        .is_ok(),
        #[cfg(feature = "postgres")]

        Database::Postgres(pool) => sqlx::query(
            "UPDATE personal_access_tokens 
                 SET name = $1, abilities = $2, updated_at = NOW()
                 WHERE id = $3",
        )
        .bind(&name)
        .bind(&abilities_json)
        .bind(id)
        .execute(pool)
        .await
        .is_ok(),
    };

    if success {
        Redirect::to("/admin/api-tokens?updated=1").into_response()
    } else {
        Redirect::to("/admin/api-tokens?error=1").into_response()
    }
}

#[derive(Template)]
#[template(path = "pages/admin.html")]
pub struct AdminDashboardTemplate {
    stats: AdminStats,
    latest_result: Option<SpeedTestResult>,
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

#[derive(Template)]
#[template(path = "pages/run-test.html")]
pub struct RunTestTemplate {
    servers: Vec<crate::api::OoklaServer>,
}

pub async fn run_test_page() -> RunTestTemplate {
    // Fetch server list (can be cached in production)
    let servers = crate::api::fetch_ookla_servers().await.unwrap_or_default();

    RunTestTemplate {
        servers: servers.into_iter().take(50).collect(), // Limit to top 50
    }
}

#[derive(serde::Deserialize)]
pub struct RunTestForm {
    server_id: Option<String>,
}

pub async fn run_test_execute(
    State(state): State<AppState>,
    Form(form): Form<RunTestForm>,
) -> Response {
    use axum::Json;
    use axum::http::StatusCode;

    // Parse server_id
    let server_id = form
        .server_id
        .and_then(|s| if s.is_empty() { None } else { Some(s) })
        .and_then(|s| s.parse::<i64>().ok());

    tracing::info!("Manual speedtest requested with server_id: {:?}", server_id);

    // Run speedtest
    let result = match crate::speedtest::run_speedtest(server_id).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Speedtest execution failed: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": e
                })),
            )
                .into_response();
        }
    };

    // Save to database
    let result_id = match crate::speedtest::save_result(&state.db, result, false).await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("Failed to save speedtest result: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Test completed but failed to save: {}", e)
                })),
            )
                .into_response();
        }
    };

    // Fetch the saved result to return
    let saved_result = match &state.db {
        #[cfg(feature = "sqlite")]

        Database::Sqlite(pool) => {
            sqlx::query_as::<_, SpeedTestResult>("SELECT * FROM results WHERE id = ?")
                .bind(result_id)
                .fetch_one(pool)
                .await
        }
        #[cfg(feature = "mysql")]

        Database::MySql(pool) => {
            sqlx::query_as::<_, SpeedTestResult>("SELECT * FROM results WHERE id = ?")
                .bind(result_id)
                .fetch_one(pool)
                .await
        }
        #[cfg(feature = "postgres")]

        Database::Postgres(pool) => {
            sqlx::query_as::<_, SpeedTestResult>("SELECT * FROM results WHERE id = $1")
                .bind(result_id)
                .fetch_one(pool)
                .await
        }
    };

    match saved_result {
        Ok(result) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "id": result.id,
                "download_mbps": format!("{:.2}", result.download_mbps()),
                "upload_mbps": format!("{:.2}", result.upload_mbps()),
                "ping": format!("{:.1}", result.ping.unwrap_or(0.0))
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch saved result: {}", e);
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "id": result_id,
                    "message": "Test completed successfully"
                })),
            )
                .into_response()
        }
    }
}
