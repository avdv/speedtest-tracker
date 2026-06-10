use axum::{
    body::Body,
    http::{header, HeaderValue, Response, StatusCode},
    response::IntoResponse,
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "public/css"]
#[include = "rust-app.css"]
pub struct CssAssets;

#[derive(RustEmbed)]
#[folder = "public"]
#[include = "favicon.ico"]
pub struct FaviconAsset;

pub async fn serve_css(path: axum::extract::Path<String>) -> impl IntoResponse {
    serve_embedded_file::<CssAssets>(&path)
}

pub async fn serve_favicon() -> impl IntoResponse {
    serve_embedded_file::<FaviconAsset>("favicon.ico")
}

fn serve_embedded_file<T>(path: &str) -> Response<Body> 
where
    T: rust_embed::RustEmbed,
{
    match T::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            let mut builder = Response::builder()
                .status(StatusCode::OK)
                .header(
                    header::CONTENT_TYPE,
                    HeaderValue::from_str(mime.as_ref()).unwrap(),
                );
            
            // Add ETag based on hash
            let hash = hex::encode(content.metadata.sha256_hash());
            builder = builder.header(
                header::ETAG,
                HeaderValue::from_str(&format!("\"{}\"", hash)).unwrap(),
            );
            
            // Add cache-control headers
            // Embedded assets are immutable, so we can cache aggressively
            builder = builder.header(
                header::CACHE_CONTROL,
                HeaderValue::from_static("public, max-age=31536000, immutable"),
            );
            
            // Add Last-Modified if available
            if let Some(last_modified) = content.metadata.last_modified() {
                if let Some(system_time) = std::time::UNIX_EPOCH.checked_add(std::time::Duration::from_secs(last_modified)) {
                    let datetime = httpdate::fmt_http_date(system_time);
                    builder = builder.header(
                        header::LAST_MODIFIED,
                        HeaderValue::from_str(&datetime).unwrap(),
                    );
                }
            }
            
            builder.body(Body::from(content.data)).unwrap()
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap(),
    }
}
