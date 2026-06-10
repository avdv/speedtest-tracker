/// Askama template filters for translations and formatting
use askama::Result;
use rust_i18n::t;
use chrono::NaiveDateTime;

/// Translate a key using the current locale
/// Usage: {{ "general.save"|t }}
pub fn t(key: &str) -> Result<String> {
    Ok(t!(key).to_string())
}

/// Translate with fallback if key not found
/// Usage: {{ "some.key"|t_or("Fallback text") }}
pub fn t_or(key: &str, fallback: &str) -> Result<String> {
    let result = t!(key);
    if result == key {
        // Key not found, use fallback
        Ok(fallback.to_string())
    } else {
        Ok(result.to_string())
    }
}

/// Format a NaiveDateTime as ISO 8601 for client-side formatting
/// Usage: {{ some_date|iso_datetime }}
pub fn iso_datetime(dt: &NaiveDateTime) -> Result<String> {
    Ok(dt.format("%Y-%m-%dT%H:%M:%S").to_string())
}

/// Translate status value (completed, failed, etc.)
/// Usage: {{ result.status|translate_status }}
pub fn translate_status(status: &str) -> Result<String> {
    let key = format!("status.{}", status.to_lowercase());
    Ok(t!(&key).to_string())
}
