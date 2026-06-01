use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use tower_sessions::Session;

const USER_ID_KEY: &str = "user_id";

pub async fn require_session(
    session: Session,
    request: Request,
    next: Next,
) -> Response {
    // Check if user is logged in
    match session.get::<i64>(USER_ID_KEY).await {
        Ok(Some(_user_id)) => {
            // User is authenticated, proceed
            next.run(request).await
        }
        _ => {
            // Not authenticated, redirect to login
            tracing::debug!("No valid session found, redirecting to login");
            Redirect::to("/login").into_response()
        }
    }
}

pub async fn set_user_session(session: Session, user_id: i64) -> Result<(), tower_sessions::session::Error> {
    session.insert(USER_ID_KEY, user_id).await
}

pub async fn clear_session(session: Session) -> Result<(), tower_sessions::session::Error> {
    session.flush().await
}

pub async fn get_user_id(session: Session) -> Option<i64> {
    session.get::<i64>(USER_ID_KEY).await.ok().flatten()
}
