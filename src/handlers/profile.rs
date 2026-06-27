use crate::error::{AppError, HtmlTemplate};
use crate::locale_middleware::Locale;
use crate::{db::Database, filters, AppState};
use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
    Form,
};
use serde::Deserialize;

#[derive(Template)]
#[template(path = "pages/profile.html")]
pub struct ProfileTemplate {
    pub locale: String,
    pub user: crate::models::User,
    pub message: Option<String>,
    pub is_authenticated: bool,
}

pub async fn profile_page(
    State(state): State<AppState>,
    locale: Locale,
    session: tower_sessions::Session,
) -> Result<Response, AppError> {
    // Get logged-in user from session
    let user_id = match crate::session::get_user_id(session).await {
        Some(id) => id,
        None => return Ok(Redirect::to("/login").into_response()),
    };

    let user = match &state.db {
        #[cfg(feature = "sqlite")]
        Database::Sqlite(pool) => {
            sqlx::query_as::<_, crate::models::User>("SELECT * FROM users WHERE id = ?")
                .bind(user_id)
                .fetch_optional(pool)
                .await?
        }
        #[cfg(feature = "mysql")]
        Database::MySql(pool) => {
            sqlx::query_as::<_, crate::models::User>("SELECT * FROM users WHERE id = ?")
                .bind(user_id)
                .fetch_optional(pool)
                .await?
        }
        #[cfg(feature = "postgres")]
        Database::Postgres(pool) => {
            sqlx::query_as::<_, crate::models::User>("SELECT * FROM users WHERE id = $1")
                .bind(user_id)
                .fetch_optional(pool)
                .await?
        }
    };

    match user {
        Some(user) => {
            let template = ProfileTemplate {
                locale: locale.0,
                user,
                message: None,
                is_authenticated: true,
            };
            Ok(HtmlTemplate(template).into_response())
        }
        None => Ok(Redirect::to("/login").into_response()),
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
) -> Result<impl IntoResponse, AppError> {
    // TODO: Get actual user ID from session
    // For now, update first admin user

    #[allow(unreachable_patterns)]
    let success = match &state.db {
        #[cfg(feature = "sqlite")]
        Database::Sqlite(pool) => {
            let result = if let Some(password) = form.password {
                if password.is_empty() {
                    sqlx::query(
                        "UPDATE users SET name = ?, email = ?, updated_at = datetime('now') 
                         WHERE role = 'admin' LIMIT 1",
                    )
                    .bind(&form.name)
                    .bind(&form.email)
                    .execute(pool)
                    .await
                } else {
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
        _ => return Ok(Redirect::to("/admin/profile")),
    };

    if success {
        Ok(Redirect::to("/admin/profile?updated=1"))
    } else {
        Ok(Redirect::to("/admin/profile?error=1"))
    }
}
