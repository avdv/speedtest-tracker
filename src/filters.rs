/// Askama template filters for translations and formatting
use askama::Result;
use rust_i18n::t;

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
