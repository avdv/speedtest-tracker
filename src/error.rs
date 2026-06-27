use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

/// Application error type. Wraps `anyhow::Error` and converts it to an HTTP
/// response. All errors are returned as JSON so they work consistently for
/// both API and web handler callers.
pub struct AppError(StatusCode, anyhow::Error);

impl AppError {
    pub fn not_found(msg: impl std::fmt::Display) -> Self {
        AppError(StatusCode::NOT_FOUND, anyhow::anyhow!("{}", msg))
    }

    pub fn bad_request(msg: impl std::fmt::Display) -> Self {
        AppError(StatusCode::BAD_REQUEST, anyhow::anyhow!("{}", msg))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.0;
        tracing::error!("HTTP {}: {:?}", status, self.1);
        let body = serde_json::json!({ "message": self.1.to_string() });
        (status, axum::response::Json(body)).into_response()
    }
}

/// Convert any `anyhow`-compatible error into an `AppError` with status 500.
impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(err: E) -> Self {
        AppError(StatusCode::INTERNAL_SERVER_ERROR, err.into())
    }
}

/// Wraps an Askama template and renders it to an HTML response.
/// On render failure it falls back to `AppError` (500).
pub struct HtmlTemplate<T>(pub T);

impl<T: Template> IntoResponse for HtmlTemplate<T> {
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => AppError::from(err).into_response(),
        }
    }
}
