use crate::locale_middleware::Locale;
use crate::{db::Database, filters, AppState};
use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};
use serde::Deserialize;

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    locale: String,
    error: Option<String>,
}

pub async fn login_page(locale: Locale) -> Response {
    let template = LoginTemplate {
        locale: locale.0,
        error: None,
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

#[derive(Deserialize)]
pub struct LoginForm {
    email: String,
    password: String,
}

pub async fn login_post(
    State(state): State<AppState>,
    session: tower_sessions::Session,
    locale: Locale,
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

                let redirect_url = session
                    .get::<String>("redirect_after_login")
                    .await
                    .ok()
                    .flatten()
                    .unwrap_or_else(|| "/".to_string());

                let _ = session.remove::<String>("redirect_after_login").await;

                if let Err(e) = crate::session::set_user_session(session, user.id).await {
                    tracing::error!("Failed to set session: {}", e);
                    let template = LoginTemplate {
                        locale: locale.0.clone(),
                        error: Some(format!("Login failed - session error: {}", e)),
                    };
                    return match template.render() {
                        Ok(html) => Html(html).into_response(),
                        Err(err) => (
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            err.to_string(),
                        )
                            .into_response(),
                    };
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

    let template = LoginTemplate {
        locale: locale.0,
        error: Some("Invalid credentials".to_string()),
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

pub async fn logout(session: tower_sessions::Session) -> Response {
    if let Err(e) = crate::session::clear_session(session).await {
        tracing::error!("Failed to clear session: {}", e);
    }
    Redirect::to("/").into_response()
}
