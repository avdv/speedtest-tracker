#[cfg(feature = "mysql")]
use sqlx::MySql;
use sqlx::Pool;
#[cfg(feature = "postgres")]
use sqlx::Postgres;
#[cfg(feature = "sqlite")]
use sqlx::Sqlite;

#[cfg(not(any(feature = "sqlite", feature = "mysql", feature = "postgres")))]
compile_error!(
    "At least one of the features \"sqlite\", \"mysql\", or \"postgres\" must be enabled."
);

#[derive(Clone)]
pub enum Database {
    #[cfg(feature = "sqlite")]
    Sqlite(Pool<Sqlite>),
    #[cfg(feature = "mysql")]
    MySql(Pool<MySql>),
    #[cfg(feature = "postgres")]
    Postgres(Pool<Postgres>),
}

impl Database {
    pub async fn connect(database_url: &str) -> Result<Self, sqlx::Error> {
        #[cfg(feature = "sqlite")]
        if database_url.starts_with("sqlite") {
            let pool = sqlx::sqlite::SqlitePoolOptions::new()
                .max_connections(5)
                .connect(&database_url)
                .await?;
            return Ok(Database::Sqlite(pool));
        }

        #[cfg(feature = "mysql")]
        if database_url.starts_with("mysql") {
            let pool = sqlx::mysql::MySqlPoolOptions::new()
                .max_connections(5)
                .connect(&database_url)
                .await?;
            return Ok(Database::MySql(pool));
        }

        #[cfg(feature = "postgres")]
        if database_url.starts_with("postgres") {
            let pool = sqlx::postgres::PgPoolOptions::new()
                .max_connections(5)
                .connect(&database_url)
                .await?;
            return Ok(Database::Postgres(pool));
        }

        panic!(
            "Database URL '{}' not supported or feature not enabled. Enable the corresponding feature: sqlite, mysql, or postgres",
            database_url
        );
    }

    pub async fn run_migrations(&self) -> Result<(), sqlx::Error> {
        match self {
            #[cfg(feature = "sqlite")]
            Database::Sqlite(pool) => {
                sqlx::migrate!("./migrations").run(pool).await?;
            }
            #[cfg(feature = "mysql")]
            Database::MySql(pool) => {
                sqlx::migrate!("./migrations").run(pool).await?;
            }
            #[cfg(feature = "postgres")]
            Database::Postgres(pool) => {
                sqlx::migrate!("./migrations").run(pool).await?;
            }
        }
        Ok(())
    }
}
