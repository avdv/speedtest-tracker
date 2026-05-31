use askama_axum::Template;
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
    Form,
};
use crate::{models::{Result as SpeedTestResult, PersonalAccessToken}, db::Database, AppState};
use serde::Deserialize;

#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {
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

pub async fn dashboard(
    State(state): State<AppState>,
    Query(params): Query<Pagination>,
) -> DashboardTemplate {
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

    DashboardTemplate {
        results,
        page: params.page,
        per_page: params.per_page,
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
}

pub async fn api_tokens_page(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> ApiTokensTemplate {
    let message = if params.contains_key("created") {
        Some("Token created successfully!".to_string())
    } else if params.contains_key("deleted") {
        Some("Token deleted successfully!".to_string())
    } else {
        None
    };

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
        new_token: None,
    }
}

#[derive(Deserialize)]
pub struct CreateTokenForm {
    name: String,
}

pub async fn create_token(
    State(state): State<AppState>,
    Form(form): Form<CreateTokenForm>,
) -> Response {
    use rand::Rng;
    
    // Generate random token
    let token: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();
    
    // Hash the token for storage
    let hashed = format!("{:x}", md5::compute(&token));
    
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
            .bind("[\"results:read\"]")
            .execute(pool)
            .await
        },
        _ => return Redirect::to("/admin/api-tokens").into_response(),
    };

    if result.is_ok() {
        // TODO: Display the token to user (only shown once!)
        Redirect::to("/admin/api-tokens?created=1").into_response()
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
