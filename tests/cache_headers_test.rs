use axum::http::StatusCode;
use speedtest_tracker::embedded_assets;

#[tokio::test]
async fn test_css_cache_headers() {
    let response =
        embedded_assets::serve_css(axum::extract::Path("rust-app.css".to_string())).await;

    // Convert IntoResponse to Response
    use axum::response::IntoResponse;
    let response = response.into_response();

    assert_eq!(response.status(), StatusCode::OK);

    // Check ETag header is present
    let etag = response.headers().get("etag");
    assert!(etag.is_some(), "ETag header should be present");
    let etag_value = etag.unwrap().to_str().unwrap();
    assert!(etag_value.starts_with("\""), "ETag should be quoted");
    assert!(etag_value.ends_with("\""), "ETag should be quoted");
    assert!(etag_value.len() > 2, "ETag should contain hash value");

    // Check Cache-Control header
    let cache_control = response.headers().get("cache-control");
    assert!(
        cache_control.is_some(),
        "Cache-Control header should be present"
    );
    let cache_value = cache_control.unwrap().to_str().unwrap();
    assert!(
        cache_value.contains("public"),
        "Should be publicly cacheable"
    );
    assert!(
        cache_value.contains("max-age=31536000"),
        "Should cache for 1 year"
    );
    assert!(
        cache_value.contains("immutable"),
        "Should be marked as immutable"
    );

    // Check Content-Type header
    let content_type = response.headers().get("content-type");
    assert!(
        content_type.is_some(),
        "Content-Type header should be present"
    );
    assert_eq!(content_type.unwrap().to_str().unwrap(), "text/css");
}

#[tokio::test]
async fn test_favicon_cache_headers() {
    let response = embedded_assets::serve_favicon().await;

    use axum::response::IntoResponse;
    let response = response.into_response();

    assert_eq!(response.status(), StatusCode::OK);

    // Check ETag is present
    let etag = response.headers().get("etag");
    assert!(etag.is_some(), "ETag header should be present for favicon");

    // Check Cache-Control
    let cache_control = response.headers().get("cache-control");
    assert!(
        cache_control.is_some(),
        "Cache-Control header should be present for favicon"
    );

    // Check Content-Type
    let content_type = response.headers().get("content-type");
    assert!(
        content_type.is_some(),
        "Content-Type header should be present"
    );
    // favicon can be either image/x-icon or image/vnd.microsoft.icon
    let ct = content_type.unwrap().to_str().unwrap();
    assert!(
        ct == "image/x-icon" || ct == "image/vnd.microsoft.icon",
        "Content-Type should be a valid icon MIME type, got: {ct}"
    );
}

#[tokio::test]
async fn test_nonexistent_file_no_cache_headers() {
    let response =
        embedded_assets::serve_css(axum::extract::Path("nonexistent.css".to_string())).await;

    use axum::response::IntoResponse;
    let response = response.into_response();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // 404 responses should not have cache headers
    let etag = response.headers().get("etag");
    assert!(etag.is_none(), "404 response should not have ETag");
}
