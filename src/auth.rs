use axum::{
    extract::{FromRequestParts, Request, State},
    http::{request::Parts, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{db::Database, models::PersonalAccessToken, AppState};

#[derive(Serialize)]
pub struct ErrorResponse {
    pub message: String,
}

impl<S> FromRequestParts<S> for PersonalAccessToken
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<PersonalAccessToken>()
            .cloned()
            .ok_or((
                StatusCode::INTERNAL_SERVER_ERROR,
                "No access token found in request extensions",
            ))
    }
}

pub async fn require_auth(
    State(state): State<AppState>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    let token = auth.token();

    tracing::debug!("Auth token received, length: {}", token.len());

    // Hash the token using SHA-256 (same as Laravel Sanctum)
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let hash_bytes = hasher.finalize();
    let hashed = hex::encode(hash_bytes);

    tracing::debug!(
        "Auth attempt - token length: {}, hash: {}...",
        token.len(),
        &hashed[..20]
    );

    // Verify token exists in database
    let token_valid = match &state.db {
        #[cfg(feature = "sqlite")]
        Database::Sqlite(pool) => sqlx::query_as::<_, PersonalAccessToken>(
            "SELECT * FROM personal_access_tokens WHERE token = $1",
        )
        .bind(&hashed)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database query error: {}", e);
            e
        })
        .ok()
        .flatten(),
        #[cfg(feature = "postgres")]
        Database::Postgres(pool) => sqlx::query_as::<_, PersonalAccessToken>(
            "SELECT * FROM personal_access_tokens WHERE token = $1",
        )
        .bind(&hashed)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database query error: {}", e);
            e
        })
        .ok()
        .flatten(),
        #[cfg(feature = "mysql")]
        Database::MySql(pool) => sqlx::query_as::<_, PersonalAccessToken>(
            "SELECT * FROM personal_access_tokens WHERE token = ?",
        )
        .bind(&hashed)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database query error: {}", e);
            e
        })
        .ok()
        .flatten(),
    };

    if token_valid.is_none() {
        tracing::warn!("Token not found in database - hash: {}...", &hashed[..20]);
    }

    if let Some(pat) = token_valid {
        // Check if token is expired
        if let Some(expires_at) = pat.expires_at {
            let now = chrono::Local::now().naive_local();
            if now > expires_at {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(ErrorResponse {
                        message: "Unauthenticated.".to_string(),
                    }),
                ));
            }
        }

        // Update last_used_at timestamp (fire and forget)
        let state_clone = state.clone();
        let token_id = pat.id;
        tokio::spawn(async move {
            match &state_clone.db {
                #[cfg(feature = "sqlite")]
                Database::Sqlite(pool) => {
                    let _ = sqlx::query(
                        "UPDATE personal_access_tokens SET last_used_at = datetime('now') WHERE id = ?"
                    )
                    .bind(token_id)
                    .execute(pool)
                    .await;
                }
                #[cfg(feature = "postgres")]
                Database::Postgres(pool) => {
                    let _ = sqlx::query(
                        "UPDATE personal_access_tokens SET last_used_at = NOW() WHERE id = ?",
                    )
                    .bind(token_id)
                    .execute(pool)
                    .await;
                }
                #[cfg(feature = "mysql")]
                Database::MySql(pool) => {
                    let _ = sqlx::query(
                        "UPDATE personal_access_tokens SET last_used_at = NOW() WHERE id = $1",
                    )
                    .bind(token_id)
                    .execute(pool)
                    .await;
                }
            };
        });

        // Store token in request extensions for later use if needed
        request.extensions_mut().insert(pat);

        Ok(next.run(request).await)
    } else {
        Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                message: "Unauthenticated.".to_string(),
            }),
        ))
    }
}
