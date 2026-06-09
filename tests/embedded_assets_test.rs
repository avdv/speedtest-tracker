use speedtest_admin::embedded_assets::{CssAssets, JsAssets, FontsAssets, FaviconAsset};

#[test]
fn test_css_assets_embedded() {
    let assets: Vec<_> = CssAssets::iter().collect();
    assert!(!assets.is_empty(), "CSS assets should be embedded");
    
    // Check for rust-app.css
    let rust_app_css = CssAssets::get("rust-app.css");
    assert!(rust_app_css.is_some(), "rust-app.css should be embedded");
}

#[test]
fn test_js_assets_embedded() {
    let assets: Vec<_> = JsAssets::iter().collect();
    assert!(!assets.is_empty(), "JS assets should be embedded");
}

#[test]
fn test_fonts_assets_embedded() {
    let assets: Vec<_> = FontsAssets::iter().collect();
    assert!(!assets.is_empty(), "Font assets should be embedded");
}

#[test]
fn test_favicon_embedded() {
    let favicon = FaviconAsset::get("favicon.ico");
    assert!(favicon.is_some(), "favicon.ico should be embedded");
    
    let favicon_data = favicon.unwrap();
    assert!(!favicon_data.data.is_empty(), "favicon.ico should have content");
}
