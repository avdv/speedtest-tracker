// Utility to create a test user in the database
// Only compiled when sqlite feature is enabled
#![cfg(feature = "sqlite")]
#![warn(
    clippy::uninlined_format_args,
    clippy::unreadable_literal,
    clippy::unused_async,
    clippy::manual_let_else,
    clippy::match_same_arms,
)]

use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let email = args
        .get(1)
        .map_or("admin@example.com", std::string::String::as_str);
    let password = args.get(2).map_or("password", std::string::String::as_str);
    let name = args.get(3).map_or("Admin", std::string::String::as_str);
    let role = args.get(4).map_or("admin", std::string::String::as_str);

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./database/database.sqlite".to_string());

    println!("Creating test user...");
    println!("  Email: {email}");
    println!("  Password: {password}");
    println!("  Name: {name}");
    println!("  Role: {role}");
    println!("  Database: {database_url}");
    println!();

    // Hash password
    let hashed = bcrypt::hash(password, 12)?;

    // Connect to database
    if database_url.starts_with("sqlite") {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await?;

        // Create tables
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                email TEXT NOT NULL UNIQUE,
                password TEXT NOT NULL,
                role TEXT NOT NULL DEFAULT 'user',
                email_verified_at DATETIME,
                remember_token TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS results (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                service TEXT DEFAULT 'ookla',
                ping REAL,
                download INTEGER,
                upload INTEGER,
                comments TEXT,
                data TEXT,
                status TEXT,
                scheduled INTEGER DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .execute(&pool)
        .await?;

        // Insert or replace user
        sqlx::query(
            "INSERT OR REPLACE INTO users (name, email, password, role, created_at, updated_at)
             VALUES (?, ?, ?, ?, datetime('now'), datetime('now'))",
        )
        .bind(name)
        .bind(email)
        .bind(hashed)
        .bind(role)
        .execute(&pool)
        .await?;

        // Add some sample results if table is empty
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM results")
            .fetch_one(&pool)
            .await?;

        if count == 0 {
            println!("Adding sample speedtest results...");
            sqlx::query(
                "INSERT INTO results (service, ping, download, upload, status, scheduled, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, datetime('now', ?), datetime('now', ?))"
            )
            .bind("ookla")
            .bind(15.2)
            .bind(95_000_000_i64)
            .bind(45_000_000_i64)
            .bind("completed")
            .bind(true)
            .bind("-1 hour")
            .bind("-1 hour")
            .execute(&pool)
            .await?;

            sqlx::query(
                "INSERT INTO results (service, ping, download, upload, status, scheduled, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, datetime('now', ?), datetime('now', ?))"
            )
            .bind("ookla")
            .bind(14.8)
            .bind(98_000_000_i64)
            .bind(47_000_000_i64)
            .bind("completed")
            .bind(true)
            .bind("-2 hours")
            .bind("-2 hours")
            .execute(&pool)
            .await?;

            sqlx::query(
                "INSERT INTO results (service, ping, download, upload, status, scheduled, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, datetime('now', ?), datetime('now', ?))"
            )
            .bind("ookla")
            .bind(16.1)
            .bind(92_000_000_i64)
            .bind(43_000_000_i64)
            .bind("completed")
            .bind(false)
            .bind("-3 hours")
            .bind("-3 hours")
            .execute(&pool)
            .await?;
        }

        println!("✅ User created successfully!");
        println!();
        println!("Login with:");
        println!("  Email: {email}");
        println!("  Password: {password}");
        println!();
        println!("Start the server:");
        println!("  cargo run --release");
        println!();
        println!("Then visit: http://localhost:3000/login");
    } else {
        println!("⚠️  Only SQLite is supported by this utility");
        println!("For MySQL/PostgreSQL, create the user manually.");
    }

    Ok(())
}
