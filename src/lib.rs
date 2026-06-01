// Library exports for testing
pub mod auth;
pub mod db;
pub mod handlers;
pub mod models;
pub mod api;
pub mod session;
pub mod speedtest;

// Re-export Database for convenience
pub use db::Database;

use axum::{
    routing::{get, post},
    Router,
    middleware,
};
use tower_http::trace::TraceLayer;
use tower_http::services::ServeDir;

#[derive(Clone)]
pub struct AppState {
    pub db: db::Database,
}

/// Create the application router with all routes configured
pub async fn create_app(state: AppState) -> Router {
    // Admin routes requiring session authentication
    let admin_routes = Router::new()
        .route("/admin", get(handlers::admin_dashboard))
        .route("/admin/results", get(handlers::results_list))
        .route("/admin/results/delete", post(handlers::delete_results))
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

    // Build main router
    Router::new()
        .route("/", get(handlers::home_dashboard))
        .route("/login", get(handlers::login_page))
        .route("/login", post(handlers::login_post))
        .route("/logout", get(handlers::logout))
        .merge(admin_routes)
        .route("/api/healthcheck", get(api::healthcheck))
        .route("/api/speedtest/latest", get(api::legacy_latest))
        .nest("/api/v1", api_v1_routes)
        .nest_service("/css", ServeDir::new("public/css"))
        .nest_service("/js", ServeDir::new("public/js"))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}
