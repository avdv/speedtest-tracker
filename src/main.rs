mod auth;
mod db;
mod handlers;
mod models;
mod api;
mod session;
mod speedtest;

use axum::{
    routing::{get, post},
    Router,
    middleware,
};
use tower_http::trace::TraceLayer;
use tower_http::services::ServeDir;
use tower_sessions::{SessionManagerLayer, Expiry};
use tower_sessions_sqlx_store::SqliteStore;
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
    let state = AppState { db: db.clone() };

    // Set up session store
    let session_store = match &db {
        db::Database::Sqlite(pool) => {
            SqliteStore::new(pool.clone())
        }
        _ => panic!("Only SQLite is supported for sessions currently"),
    };
    
    session_store.migrate().await?;
    
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false) // Set to true in production with HTTPS
        .with_expiry(Expiry::OnInactivity(time::Duration::hours(24)));

    // Admin routes requiring session authentication
    let admin_routes = Router::new()
        .route("/admin", get(handlers::admin_dashboard))
        .route("/admin/results", get(handlers::results_list))
        .route("/admin/profile", get(handlers::profile_page))
        .route("/admin/profile", post(handlers::profile_update))
        .route("/admin/api-tokens", get(handlers::api_tokens_page))
        .route("/admin/api-tokens/create", post(handlers::create_token))
        .route("/admin/api-tokens/edit/:id", get(handlers::edit_token_page))
        .route("/admin/api-tokens/update", post(handlers::update_token))
        .route("/admin/api-tokens/delete", post(handlers::delete_token))
        .route("/admin/speedtest", get(handlers::run_test_page))
        .route("/admin/speedtest/run", post(handlers::run_test_execute))
        .layer(middleware::from_fn(session::require_session));

    // API v1 routes requiring token authentication
    let api_v1_routes = Router::new()
        .route("/results", get(api::list_results))
        .route("/results/latest", get(api::latest_result))
        .route("/results/:id", get(api::get_result))
        .route("/stats", get(api::get_stats))
        .route("/ookla/list-servers", get(api::list_ookla_servers))
        .route("/speedtests/run", post(api::run_speedtest_api))
        .layer(middleware::from_fn_with_state(state.clone(), auth::require_auth));

    let app = Router::new()
        .route("/", get(handlers::home_dashboard))
        .merge(admin_routes)
        .route("/login", get(handlers::login_page))
        .route("/login", post(handlers::login_post))
        .route("/logout", get(handlers::logout))
        // Public API routes
        .route("/api/healthcheck", get(api::healthcheck))
        .route("/api/speedtest/latest", get(api::legacy_latest))
        // Protected API v1 routes
        .nest("/api/v1", api_v1_routes)
        // Static file serving
        .nest_service("/css", ServeDir::new("public/css"))
        .nest_service("/js", ServeDir::new("public/js"))
        .nest_service("/fonts", ServeDir::new("public/fonts"))
        .layer(session_layer)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    
    tracing::info!("Starting server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
