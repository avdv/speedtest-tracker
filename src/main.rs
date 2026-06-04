mod api;
mod auth;
mod db;
pub mod filters;
mod handlers;
mod i18n;
mod locale_middleware;
mod models;
mod scheduler;
mod session;
mod speedtest;

rust_i18n::i18n!("locales", fallback = "en");

use axum::{
    Router, middleware,
    routing::{get, post},
};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tower_sessions::{Expiry, SessionManagerLayer};
#[cfg(feature = "mysql")]
use tower_sessions_sqlx_store::MySqlStore;
#[cfg(feature = "postgres")]
use tower_sessions_sqlx_store::PostgresStore;
#[cfg(feature = "sqlite")]
use tower_sessions_sqlx_store::SqliteStore;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Make public for tests
pub use db::Database;

#[derive(Clone)]
pub struct AppState {
    pub db: db::Database,
}

// Wrapper enum for different session store types
#[derive(Clone, Debug)]
pub enum AnySessionStore {
    #[cfg(feature = "sqlite")]
    Sqlite(SqliteStore),
    #[cfg(feature = "mysql")]
    MySql(MySqlStore),
    #[cfg(feature = "postgres")]
    Postgres(PostgresStore),
}

#[async_trait::async_trait]
impl tower_sessions::SessionStore for AnySessionStore {
    async fn save(
        &self,
        record: &tower_sessions::session::Record,
    ) -> tower_sessions::session_store::Result<()> {
        match self {
            #[cfg(feature = "sqlite")]
            AnySessionStore::Sqlite(store) => store.save(record).await,
            #[cfg(feature = "mysql")]
            AnySessionStore::MySql(store) => store.save(record).await,
            #[cfg(feature = "postgres")]
            AnySessionStore::Postgres(store) => store.save(record).await,
        }
    }

    async fn load(
        &self,
        session_id: &tower_sessions::session::Id,
    ) -> tower_sessions::session_store::Result<Option<tower_sessions::session::Record>> {
        match self {
            #[cfg(feature = "sqlite")]
            AnySessionStore::Sqlite(store) => store.load(session_id).await,
            #[cfg(feature = "mysql")]
            AnySessionStore::MySql(store) => store.load(session_id).await,
            #[cfg(feature = "postgres")]
            AnySessionStore::Postgres(store) => store.load(session_id).await,
        }
    }

    async fn delete(
        &self,
        session_id: &tower_sessions::session::Id,
    ) -> tower_sessions::session_store::Result<()> {
        match self {
            #[cfg(feature = "sqlite")]
            AnySessionStore::Sqlite(store) => store.delete(session_id).await,
            #[cfg(feature = "mysql")]
            AnySessionStore::MySql(store) => store.delete(session_id).await,
            #[cfg(feature = "postgres")]
            AnySessionStore::Postgres(store) => store.delete(session_id).await,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "speedtest_admin=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db = db::Database::connect().await?;

    // Run migrations
    tracing::info!("Running database migrations...");
    db.run_migrations().await?;
    tracing::info!("Database migrations completed");

    // Initialize and start the scheduler
    let scheduler = scheduler::SpeedtestScheduler::new(db.clone()).await?;
    scheduler.start().await?;

    let state = AppState { db: db.clone() };

    // Set up session store with conditional compilation
    let session_store = match &db {
        #[cfg(feature = "sqlite")]
        db::Database::Sqlite(pool) => {
            let store = SqliteStore::new(pool.clone());
            store.migrate().await?;
            AnySessionStore::Sqlite(store)
        }
        #[cfg(feature = "mysql")]
        db::Database::MySql(pool) => {
            let store = MySqlStore::new(pool.clone());
            store.migrate().await?;
            AnySessionStore::MySql(store)
        }
        #[cfg(feature = "postgres")]
        db::Database::Postgres(pool) => {
            let store = PostgresStore::new(pool.clone());
            store.migrate().await?;
            AnySessionStore::Postgres(store)
        }
    };

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false) // Set to true in production with HTTPS
        .with_expiry(Expiry::OnInactivity(time::Duration::hours(24)));

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
        .route("/results/:id", get(api::get_result))
        .route("/stats", get(api::get_stats))
        .route("/ookla/list-servers", get(api::list_ookla_servers))
        .route("/speedtests/run", post(api::run_speedtest_api))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_auth,
        ));

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
        .route(
            "/favicon.ico",
            get(|| async {
                use axum::body::Body;
                use axum::http::{StatusCode, header};
                use axum::response::Response;

                match tokio::fs::read("public/favicon.ico").await {
                    Ok(contents) => Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "image/x-icon")
                        .body(Body::from(contents))
                        .unwrap(),
                    Err(e) => {
                        tracing::error!("Error reading favicon.ico: {}", e);
                        Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .body(Body::empty())
                            .unwrap()
                    }
                }
            }),
        )
        .nest_service("/css", ServeDir::new("public/css"))
        .nest_service("/js", ServeDir::new("public/js"))
        .nest_service("/fonts", ServeDir::new("public/fonts"))
        .layer(middleware::from_fn(locale_middleware::locale_middleware))
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
