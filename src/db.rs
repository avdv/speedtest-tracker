use sqlx::{Pool, Sqlite, MySql, Postgres};
use std::env;

#[derive(Clone)]
pub enum Database {
    Sqlite(Pool<Sqlite>),
    MySql(Pool<MySql>),
    Postgres(Pool<Postgres>),
}

impl Database {
    pub async fn connect() -> Result<Self, sqlx::Error> {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite:./database/database.sqlite".to_string());

        if database_url.starts_with("sqlite") {
            let pool = sqlx::sqlite::SqlitePoolOptions::new()
                .max_connections(5)
                .connect(&database_url)
                .await?;
            Ok(Database::Sqlite(pool))
        } else if database_url.starts_with("mysql") {
            let pool = sqlx::mysql::MySqlPoolOptions::new()
                .max_connections(5)
                .connect(&database_url)
                .await?;
            Ok(Database::MySql(pool))
        } else if database_url.starts_with("postgres") {
            let pool = sqlx::postgres::PgPoolOptions::new()
                .max_connections(5)
                .connect(&database_url)
                .await?;
            Ok(Database::Postgres(pool))
        } else {
            panic!("Unsupported database type");
        }
    }
}
