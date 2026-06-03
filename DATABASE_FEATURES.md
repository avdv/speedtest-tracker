# Database Feature Flags

Speedtest Tracker Rust port supports selective database compilation via Cargo features.

## Features

- `db-sqlite` (default) - SQLite support
- `db-mysql` - MySQL/MariaDB support  
- `db-postgres` - PostgreSQL support

## Building

```bash
# Default (SQLite)
cargo build

# MySQL only
cargo build --no-default-features --features db-mysql

# PostgreSQL only
cargo build --no-default-features --features db-postgres

# All databases
cargo build --features db-sqlite,db-mysql,db-postgres
```

## Benefits

- **Smaller binary sizes** - Only compile database drivers you need
- **Faster compilation** - Less code to compile
- **Cleaner dependencies** - No unused database crates
- **Flexible deployment** - Choose database backend at build time

## How It Works

### Feature Flag Alignment

Our database features correctly enable the corresponding SQLx features:

| Our Feature   | Enables SQLx Feature | Enables Session Store |
|---------------|---------------------|----------------------|
| `db-sqlite`   | `sqlx/sqlite`       | `tower-sessions-sqlx-store/sqlite` |
| `db-mysql`    | `sqlx/mysql`        | `tower-sessions-sqlx-store/mysql` |
| `db-postgres` | `sqlx/postgres`     | `tower-sessions-sqlx-store/postgres` |

### SQLx Configuration

SQLx is configured with `default-features = false` and we explicitly enable:
- `runtime-tokio-rustls` - Async runtime with Rustls TLS
- `chrono` - DateTime support
- `macros` - Compile-time SQL checking
- `migrate` - Database migrations

This ensures we only include what we need and avoid SQLx's default features (which enable all databases).

### Runtime Detection

The database type is automatically detected from `DATABASE_URL` environment variable:

```bash
# SQLite
DATABASE_URL="sqlite:./database/database.sqlite"

# MySQL
DATABASE_URL="mysql://user:password@localhost/speedtest"

# PostgreSQL  
DATABASE_URL="postgres://user:password@localhost/speedtest"
```

## Binary Utilities

### create-test-user

Requires the `db-sqlite` feature:

```bash
cargo build --features db-sqlite --bin create-test-user
./target/debug/create-test-user admin@example.com password "Admin"
```

## Verification

Check enabled features:

```bash
# View all features
cargo tree -e features -p speedtest-admin

# Check SQLx features
cargo tree -e features -p sqlx
```
