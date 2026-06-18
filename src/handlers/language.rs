use axum::{
    extract::Path,
    http::HeaderMap,
    http::{header::REFERER, Uri},
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::{cookie::Cookie, CookieJar};

/// Handle language change requests
/// Route: GET /set-language/:locale
pub async fn set_language(
    Path(locale): Path<String>,
    headers: HeaderMap,
    cookies: CookieJar,
) -> impl IntoResponse {
    tracing::info!("Language change requested: {}", locale);

    // Normalize and validate the locale
    let normalized_locale = crate::i18n::normalize_locale(&locale);
    tracing::debug!("Normalized '{}' to '{}'", locale, normalized_locale);

    // Get the referer URL to redirect back to the current page
    let redirect_to = headers
        .get(REFERER)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<Uri>().ok())
        .and_then(|uri: Uri| uri.path_and_query().map(|pq| pq.as_str().to_string()))
        .filter(|path| !path.is_empty())
        .unwrap_or("/".to_string());

    tracing::debug!("Redirecting to: {}", redirect_to);

    if !crate::i18n::is_valid_locale(&normalized_locale) {
        tracing::warn!(
            "Invalid locale requested: {} (normalized: {})",
            locale,
            normalized_locale
        );
        // Redirect back to referer or home with default locale
        return (cookies, Redirect::to(&redirect_to));
    }

    // Create a cookie that lasts for 1 year
    let cookie = Cookie::build(("locale", normalized_locale.clone()))
        .path("/")
        .max_age(time::Duration::days(365))
        .http_only(false) // Allow JavaScript to read this cookie
        .build();

    tracing::info!("Setting locale cookie to: {}", normalized_locale);

    // Add cookie and redirect back to the referring page
    let updated_cookies = cookies.add(cookie);
    (updated_cookies, Redirect::to(&redirect_to))
}
