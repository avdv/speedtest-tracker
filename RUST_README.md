# Speedtest Tracker Admin (Rust)

A minimal Rust replacement for the Speedtest Tracker admin interface.

## Features

- ✅ Dashboard with speedtest results
- ✅ Simple authentication
- ✅ SQLite/MySQL/PostgreSQL support
- ✅ Minimal dependencies
- ✅ Cross-compile for armv7

## Prerequisites

- Rust 1.75+ (install from https://rustup.rs)
- Existing speedtest-tracker database

## Quick Start

1. **Set database connection:**
```bash
export DATABASE_URL="sqlite:./database/database.sqlite"
# or MySQL: mysql://user:pass@localhost/speedtest
# or Postgres: postgresql://user:pass@localhost/speedtest
```

2. **Run development server:**
```bash
cargo run
```

3. **Access admin panel:**
Open http://localhost:3000

## Build for armv7

```bash
# Install cross-compilation tools
rustup target add armv7-unknown-linux-gnueabihf

# Install cross (makes cross-compilation easy)
cargo install cross

# Build for armv7
cross build --release --target armv7-unknown-linux-gnueabihf

# Binary will be at: target/armv7-unknown-linux-gnueabihf/release/speedtest-admin
```

## Environment Variables

- `DATABASE_URL` - Database connection string (default: sqlite:./database/database.sqlite)
- `PORT` - Server port (default: 3000)
- `RUST_LOG` - Logging level (default: info)

## Production Deployment

```bash
# Build optimized binary
cargo build --release

# Run with systemd, supervisor, or docker
./target/release/speedtest-admin
```

## Size Comparison

- PHP + Laravel + Dependencies: ~150MB
- Rust binary (optimized): ~5-10MB
- Memory usage: ~10-20MB vs ~100-200MB for PHP

## What's Included

- Dashboard page showing speed test results
- Login page with bcrypt password verification
- Support for all three database types
- Pagination
- Clean, responsive UI

## What's Not Included (Yet)

- Settings pages
- User management UI
- Running speedtests from UI
- Notifications
- API tokens management
- Charts/graphs

These can be added incrementally as needed.
