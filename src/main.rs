mod db;
mod handlers;
mod models;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
pub struct AppState {
    db: db::Database,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "speedtest_admin=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db = db::Database::connect().await?;
    let state = AppState { db };

    let app = Router::new()
        .route("/", get(handlers::dashboard))
        .route("/admin", get(handlers::admin_dashboard))
        .route("/login", get(handlers::login_page))
        .route("/login", post(handlers::login_post))
        .route("/admin/profile", get(handlers::profile_page))
        .route("/admin/profile", post(handlers::profile_update))
        .route("/admin/api-tokens", get(handlers::api_tokens_page))
        .route("/admin/api-tokens/create", post(handlers::create_token))
        .route("/admin/api-tokens/delete", post(handlers::delete_token))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    
    tracing::info!("Starting server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
