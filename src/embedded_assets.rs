use axum::{
    body::Body,
    http::{header, HeaderValue, Response, StatusCode},
    response::IntoResponse,
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "public/css"]
#[exclude = "filament/"]
pub struct CssAssets;

#[derive(RustEmbed)]
#[folder = "public/js"]
pub struct JsAssets;

#[derive(RustEmbed)]
#[folder = "public/fonts"]
pub struct FontsAssets;

#[derive(RustEmbed)]
#[folder = "public"]
#[include = "favicon.ico"]
pub struct FaviconAsset;

pub async fn serve_css(path: axum::extract::Path<String>) -> impl IntoResponse {
    serve_embedded_file::<CssAssets>(&path)
}

pub async fn serve_js(path: axum::extract::Path<String>) -> impl IntoResponse {
    serve_embedded_file::<JsAssets>(&path)
}

pub async fn serve_fonts(path: axum::extract::Path<String>) -> impl IntoResponse {
    serve_embedded_file::<FontsAssets>(&path)
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
            
            Response::builder()
                .status(StatusCode::OK)
                .header(
                    header::CONTENT_TYPE,
                    HeaderValue::from_str(mime.as_ref()).unwrap(),
                )
                .body(Body::from(content.data))
                .unwrap()
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap(),
    }
}
