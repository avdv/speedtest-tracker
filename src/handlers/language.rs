use axum::{
    extract::Path,
    response::{Redirect, IntoResponse},
};
use axum_extra::extract::{CookieJar, cookie::Cookie};

/// Handle language change requests
/// Route: GET /set-language/:locale
pub async fn set_language(
    Path(locale): Path<String>,
    cookies: CookieJar,
) -> impl IntoResponse {
    tracing::info!("Language change requested: {}", locale);
    
    // Normalize and validate the locale
    let normalized_locale = crate::i18n::normalize_locale(&locale);
    tracing::debug!("Normalized '{}' to '{}'", locale, normalized_locale);
    
    if !crate::i18n::is_valid_locale(&normalized_locale) {
        tracing::warn!("Invalid locale requested: {} (normalized: {})", locale, normalized_locale);
        // Redirect back to home with default locale
        return (cookies, Redirect::to("/"));
    }
    
    // Create a cookie that lasts for 1 year
    let cookie = Cookie::build(("locale", normalized_locale.clone()))
        .path("/")
        .max_age(time::Duration::days(365))
        .http_only(false) // Allow JavaScript to read this cookie
        .build();
    
    tracing::info!("Setting locale cookie to: {}", normalized_locale);
    
    // Add cookie and redirect to home
    let updated_cookies = cookies.add(cookie);
    (updated_cookies, Redirect::to("/"))
}
