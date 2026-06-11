// Library exports for testing
pub mod api;
pub mod auth;
pub mod db;
pub mod embedded_assets;
pub mod filters;
pub mod handlers;
pub mod i18n;
pub mod locale_middleware;
pub mod models;
pub mod session;
pub mod scheduler;
pub mod speedtest;

rust_i18n::i18n!("locales", fallback = "en");

// Re-export Database for convenience
pub use db::Database;

use axum::{
    Router, middleware,
    routing::{get, post},
};
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub db: db::Database,
}

/// Create the application router with all routes configured
pub fn create_app(state: AppState) -> Router {
    // Admin routes requiring session authentication
    let admin_routes = Router::new()
        .route("/admin", get(handlers::admin_dashboard))
        .route("/admin/results/test", get(|| async { "Test route works!" }))
        .route("/admin/results/{id}", get(api::get_result))
        .route("/admin/results", get(handlers::results_list))
        .route("/admin/results/delete", post(handlers::delete_results))
        .route("/admin/profile", get(handlers::profile_page))
        .route("/admin/profile", post(handlers::profile_update))
        .route("/admin/api-tokens", get(handlers::api_tokens_page))
        .route("/admin/api-tokens/create", post(handlers::create_token))
        .route("/admin/api-tokens/edit/{id}", get(handlers::edit_token_page))
        .route("/admin/api-tokens/update", post(handlers::update_token))
        .route("/admin/api-tokens/delete", post(handlers::delete_token))
        .route("/admin/schedules", get(handlers::schedules_page))
        .route("/admin/schedules/create", post(handlers::create_schedule))
        .route("/admin/schedules/delete", post(handlers::delete_schedule))
        .route("/admin/schedules/toggle", post(handlers::toggle_schedule))
        .route("/admin/speedtest", get(handlers::run_test_page))
        .route("/admin/speedtest/run", post(handlers::run_test_execute))
        .layer(middleware::from_fn(session::require_session));

    // API v1 routes requiring token authentication
    let api_v1_routes = Router::new()
        .route("/results", get(api::list_results))
        .route("/results/latest", get(api::latest_result))
        .route("/results/{id}", get(api::get_result))
        .route("/stats", get(api::get_stats))
        .route("/ookla/list-servers", get(api::list_ookla_servers))
        .route("/speedtests/run", post(api::run_speedtest_api))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_auth,
        ));

    // Build main router
    Router::new()
        .route("/", get(handlers::home_dashboard))
        .merge(admin_routes)
        .route("/login", get(handlers::login_page))
        .route("/login", post(handlers::login_post))
        .route("/logout", get(handlers::logout))
        .route("/set-language/{locale}", get(handlers::set_language))
        // Public API routes
        .route("/api/healthcheck", get(api::healthcheck))
        .route("/api/speedtest/latest", get(api::legacy_latest))
        // Protected API v1 routes
        .nest("/api/v1", api_v1_routes)
        // Static file serving
        .route("/favicon.ico", get(embedded_assets::serve_favicon))
        .route("/css/{*path}", get(embedded_assets::serve_css))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}
