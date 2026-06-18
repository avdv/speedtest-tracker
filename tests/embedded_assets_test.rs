use speedtest_tracker::embedded_assets::{CssAssets, FaviconAsset};

#[test]
fn test_css_assets_embedded() {
    // Only rust-app.css should be embedded
    let rust_app_css = CssAssets::get("rust-app.css");
    assert!(rust_app_css.is_some(), "rust-app.css should be embedded");

    let css_data = rust_app_css.unwrap();
    assert!(
        !css_data.data.is_empty(),
        "rust-app.css should have content"
    );
}

#[test]
fn test_favicon_embedded() {
    let favicon = FaviconAsset::get("favicon.ico");
    assert!(favicon.is_some(), "favicon.ico should be embedded");

    let favicon_data = favicon.unwrap();
    assert!(
        !favicon_data.data.is_empty(),
        "favicon.ico should have content"
    );
}

#[test]
fn test_filament_assets_not_embedded() {
    // Verify Filament assets are NOT embedded
    let filament_css = CssAssets::get("filament/filament/app.css");
    assert!(
        filament_css.is_none(),
        "Filament CSS should not be embedded"
    );
}

#[test]
fn test_asset_metadata() {
    let css = CssAssets::get("rust-app.css").unwrap();

    // Verify hash is available
    let hash = css.metadata.sha256_hash();
    assert_eq!(hash.len(), 32, "SHA256 hash should be 32 bytes");

    // Verify last_modified is available (may be None depending on build)
    // Just checking the method exists and returns Option
    let _ = css.metadata.last_modified();
}
