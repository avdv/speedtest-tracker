use crate::locale_middleware::Locale;
use crate::{db::Database, filters, models::PersonalAccessToken, AppState};
use askama::Template;
use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};
use rand::RngExt;
use serde::Deserialize;

#[derive(Template)]
#[template(path = "pages/api-tokens.html")]
pub struct ApiTokensTemplate {
    pub locale: String,
    pub tokens: Vec<PersonalAccessToken>,
    pub message: Option<String>,
    pub new_token: Option<String>,
    pub new_token_name: Option<String>,
    pub is_authenticated: bool,
}

pub async fn api_tokens_page(
    State(state): State<AppState>,
    locale: Locale,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let message = if params.contains_key("deleted") {
        Some("Token deleted successfully!".to_string())
    } else if params.contains_key("error") {
        Some("An error occurred.".to_string())
    } else {
        None
    };

    let new_token = params.get("token").cloned();
    let new_token_name = params.get("token_name").cloned();

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

    let template = ApiTokensTemplate {
        locale: locale.0,
        tokens,
        message,
        new_token,
        new_token_name,
        is_authenticated: true,
    };

    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(err) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            err.to_string(),
        )
            .into_response(),
    }
}

pub async fn create_token(State(state): State<AppState>, body: String) -> Response {
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
    let token: String = rand::rng()
        .sample_iter(&rand::distr::Alphanumeric)
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

#[derive(Template)]
#[template(path = "pages/edit-token.html")]
pub struct EditTokenTemplate {
    locale: String,
    token: PersonalAccessToken,
    error: Option<String>,
    is_authenticated: bool,
}

pub async fn edit_token_page(
    State(state): State<AppState>,
    locale: Locale,
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
        Ok(Some(token)) => {
            let template = EditTokenTemplate {
                locale: locale.0,
                token,
                error: None,
                is_authenticated: true,
            };
            match template.render() {
                Ok(html) => Html(html).into_response(),
                Err(err) => (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    err.to_string(),
                )
                    .into_response(),
            }
        }
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
