use crate::{filters, AppState, db::Database};
use crate::locale_middleware::Locale;
use askama::Template;
use axum::{
    Form,
    extract::State,
    response::{Html, IntoResponse, Redirect, Response},
};
use serde::Deserialize;

#[derive(Template)]
#[template(path = "pages/profile.html")]
pub struct ProfileTemplate {
    pub locale: String,
    pub user: crate::models::User,
    pub message: Option<String>,
}

pub async fn profile_page(
    State(state): State<AppState>,
    locale: Locale,
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
        Ok(Some(user)) => {
            let template = ProfileTemplate {
                locale: locale.0,
                user,
                message: None,
            };
            match template.render() {
                Ok(html) => Html(html).into_response(),
                Err(err) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
            }
        },
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
