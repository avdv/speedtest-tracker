/// Middleware for locale detection and i18n setup
use axum::{
    extract::{Request, FromRequestParts},
    http::{request::Parts, HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::CookieJar;

/// Locale stored in request extensions
#[derive(Clone, Debug)]
pub struct Locale(pub String);

/// Axum extractor for Locale
#[axum::async_trait]
impl<S> FromRequestParts<S> for Locale
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Locale>()
            .cloned()
            .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Locale not found in request extensions"))
    }
}

/// Middleware that detects and stores locale in request extensions
pub async fn locale_middleware(
    cookies: CookieJar,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Response {
    // Detect locale from request (cookie first, then headers)
    let locale = detect_locale(&cookies, &headers);
    
    tracing::debug!("Detected locale for request: {}", locale);
    
    // Store locale in request extensions (thread-safe!)
    request.extensions_mut().insert(Locale(locale));
    
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
