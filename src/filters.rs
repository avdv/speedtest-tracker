/// Askama template filters for translations and formatting
use askama::Result;
use rust_i18n::t;
use chrono::NaiveDateTime;

/// Translate a key using the provided locale
/// Usage in template: {{ "general.save"|t(locale) }}
#[askama::filter_fn]
pub fn t(key: &str, _env: &dyn askama::Values, locale: &str) -> Result<String> {
    Ok(t!(key, locale = locale).to_string())
}

/// Translate with fallback if key not found
/// Usage: {{ "some.key"|t_or("Fallback text", locale) }}
#[askama::filter_fn]
pub fn t_or(key: &str, _env: &dyn askama::Values, fallback: &str, locale: &str) -> Result<String> {
    let result = t!(key, locale = locale);
    if result == key {
        // Key not found, use fallback
        Ok(fallback.to_string())
    } else {
        Ok(result.to_string())
    }
}

/// Format a NaiveDateTime as ISO 8601 for client-side formatting
/// Usage: {{ some_date|iso_datetime }}
#[askama::filter_fn]
pub fn iso_datetime(dt: &NaiveDateTime, _env: &dyn askama::Values) -> Result<String> {
    Ok(dt.format("%Y-%m-%dT%H:%M:%S").to_string())
}

/// Translate status value (completed, failed, etc.)
/// Usage: {{ result.status|translate_status(locale) }}
#[askama::filter_fn]
pub fn translate_status(status: &str, _env: &dyn askama::Values, locale: &str) -> Result<String> {
    let key = format!("status.{}", status.to_lowercase());
    Ok(t!(&key, locale = locale).to_string())
}
