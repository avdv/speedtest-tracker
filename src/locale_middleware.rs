/// Middleware for locale detection and i18n setup
use axum::{
    extract::Request,
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use axum_extra::extract::CookieJar;

/// Middleware that detects and sets locale for each request
pub async fn locale_middleware(
    cookies: CookieJar,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Response {
    // Detect locale from request (cookie first, then headers)
    let locale = detect_locale(&cookies, &headers);
    
    tracing::debug!("Setting locale to: {}", locale);
    
    // Set locale for this request's thread
    rust_i18n::set_locale(&locale);
    
    // Verify it was set
    let current_locale = rust_i18n::locale();
    tracing::debug!("Current locale is now: {}", &*current_locale);
    
    // Continue with request
    next.run(request).await
}

/// Detect locale from cookies and HTTP headers
fn detect_locale(cookies: &CookieJar, headers: &HeaderMap) -> String {
    // 1. Check cookie first (user preference)
    if let Some(cookie) = cookies.get("locale") {
        let locale = cookie.value().to_string();
        tracing::debug!("Found locale cookie: {}", locale);
        
        if crate::i18n::is_valid_locale(&locale) {
            tracing::info!("Using locale from cookie: {}", locale);
            return locale;
        }
    }
    
    // 2. Check Accept-Language header
    detect_locale_from_headers(headers)
}

/// Detect locale from HTTP headers
fn detect_locale_from_headers(headers: &HeaderMap) -> String {
    // 1. Check Accept-Language header
    if let Some(accept_lang) = headers.get("Accept-Language") {
        if let Ok(lang_str) = accept_lang.to_str() {
            tracing::debug!("Accept-Language header: {}", lang_str);
            
            // Parse format: "en-US,en;q=0.9,de;q=0.8"
            for lang_part in lang_str.split(',') {
                let lang = lang_part
                    .split(';')
                    .next()
                    .map(|l| l.trim().to_lowercase())
                    .unwrap_or_default();
                
                if !lang.is_empty() {
                    let normalized = crate::i18n::normalize_locale(&lang);
                    tracing::debug!("Testing locale: {} -> normalized: {}", lang, normalized);
                    
                    if crate::i18n::is_valid_locale(&normalized) {
                        tracing::info!("Selected locale: {}", normalized);
                        return normalized;
                    }
                }
            }
        }
    } else {
        tracing::debug!("No Accept-Language header found");
    }
    
    // 2. Fallback to English
    tracing::info!("Using fallback locale: en");
    "en".to_string()
}
