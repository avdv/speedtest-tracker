/// This module provides i18n utilities for the speedtest-tracker application

/// Normalize locale string to match available translations
pub fn normalize_locale(lang: &str) -> String {
    let lower = lang.to_lowercase();
    match lower.as_str() {
        "en" | "en-us" | "en-gb" | "en-ca" | "en-au" => "en".to_string(),
        "de" | "de-de" | "de-at" | "de-ch" => "de_DE".to_string(),
        "es" | "es-es" | "es-mx" | "es-ar" => "es_ES".to_string(),
        "fr" | "fr-fr" | "fr-ca" | "fr-be" => "fr_FR".to_string(),
        "nl" | "nl-nl" | "nl-be" => "nl_NL".to_string(),
        "pt-br" | "pt_br" => "pt_BR".to_string(),
        _ => "en".to_string(),
    }
}

/// Check if a locale is supported
pub fn is_valid_locale(locale: &str) -> bool {
    matches!(locale, "en" | "de_DE" | "es_ES" | "fr_FR" | "nl_NL" | "pt_BR")
}

/// Get list of all available locales with their display names
pub fn available_locales() -> Vec<(&'static str, &'static str)> {
    vec![
        ("en", "English"),
        ("de_DE", "Deutsch"),
        ("es_ES", "Español"),
        ("fr_FR", "Français"),
        ("nl_NL", "Nederlands"),
        ("pt_BR", "Português (Brasil)"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_locale() {
        assert_eq!(normalize_locale("en"), "en");
        assert_eq!(normalize_locale("en-US"), "en");
        assert_eq!(normalize_locale("de"), "de_DE");
        assert_eq!(normalize_locale("DE-DE"), "de_DE");
        assert_eq!(normalize_locale("pt-BR"), "pt_BR");
        assert_eq!(normalize_locale("unknown"), "en");
    }

    #[test]
    fn test_is_valid_locale() {
        assert!(is_valid_locale("en"));
        assert!(is_valid_locale("de_DE"));
        assert!(is_valid_locale("pt_BR"));
        assert!(!is_valid_locale("jp"));
        assert!(!is_valid_locale("unknown"));
    }
}
