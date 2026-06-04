/// Middleware for locale detection and i18n setup
use axum::{
    extract::Request,
    http::HeaderMap,
    middleware::Next,
    response::Response,
};

/// Middleware that detects and sets locale for each request
pub async fn locale_middleware(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Response {
    // Detect locale from request
    let locale = detect_locale_from_headers(&headers);
    
    // Set locale for this request's thread
    rust_i18n::set_locale(&locale);
    
    // Continue with request
    next.run(request).await
}

/// Detect locale from HTTP headers
fn detect_locale_from_headers(headers: &HeaderMap) -> String {
    // 1. Check Accept-Language header
    if let Some(accept_lang) = headers.get("Accept-Language") {
        if let Ok(lang_str) = accept_lang.to_str() {
            // Parse format: "en-US,en;q=0.9,de;q=0.8"
            for lang_part in lang_str.split(',') {
                let lang = lang_part
                    .split(';')
                    .next()
                    .map(|l| l.trim().to_lowercase())
                    .unwrap_or_default();
                
                if !lang.is_empty() {
                    let normalized = crate::i18n::normalize_locale(&lang);
                    if crate::i18n::is_valid_locale(&normalized) {
                        return normalized;
                    }
                }
            }
        }
    }
    
    // 2. Fallback to English
    "en".to_string()
}
