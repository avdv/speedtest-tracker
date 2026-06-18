// Import all modules from the library
use speedtest_tracker::{AppState, db, locale_middleware, scheduler};

rust_i18n::i18n!("locales", fallback = "en");

use axum::middleware;
use clap::Parser;
use tower_sessions::{Expiry, SessionManagerLayer};
#[cfg(feature = "mysql")]
use tower_sessions_sqlx_store::MySqlStore;
#[cfg(feature = "postgres")]
use tower_sessions_sqlx_store::PostgresStore;
#[cfg(feature = "sqlite")]
use tower_sessions_sqlx_store::SqliteStore;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(
    version,
    about = "Speedtest Tracker — self-hosted internet performance monitoring"
)]
struct Cli {
    /// Port to listen on
    #[arg(short, long, env = "PORT", default_value = "3000")]
    port: u16,
    #[arg(short, long, env = "DATABASE_URL")]
    database_url: Option<String>,
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

    let cli = Cli::parse();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "speedtest_tracker=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    #[cfg(feature = "sqlite")]
    let database_url = cli
        .database_url
        .unwrap_or("sqlite:./database/database.sqlite".to_string());
    #[cfg(not(feature = "sqlite"))]
    let database_url = cli.database_url.expect("DATABASE_URL must be configured")?;

    let db = db::Database::connect(&database_url).await?;

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

    // Create the application router using the centralized configuration from lib.rs
    let app = speedtest_tracker::create_app(state)
        .layer(middleware::from_fn(locale_middleware::locale_middleware))
        .layer(session_layer);

    let addr = format!("0.0.0.0:{}", cli.port);

    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
