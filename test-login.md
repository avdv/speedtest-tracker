# Test Login Credentials

## Quick Setup

Run this command to create a test database with a user and sample data:

```bash
cargo run --bin create-test-user
```

This creates:
- **Database:** `database/database.sqlite`
- **User:** admin@example.com
- **Password:** password
- **Sample speedtest results** for testing

## Custom User

Create your own user:

```bash
cargo run --bin create-test-user your@email.com yourpassword "Your Name" admin
```

## Start the Server

```bash
export DATABASE_URL=sqlite:./database/database.sqlite
cargo run --release
```

Then visit: **http://localhost:3000/login**

## Default Test Credentials

**Email:** `admin@example.com`  
**Password:** `password`

## Using Existing Database

If you already have a speedtest-tracker database from the PHP version, just point to it:

```bash
export DATABASE_URL=sqlite:/path/to/existing/database.sqlite
# or
export DATABASE_URL=mysql://user:pass@localhost/speedtest_tracker
# or  
export DATABASE_URL=postgresql://user:pass@localhost/speedtest_tracker

cargo run --release
```

The Rust server will use the existing users from that database.
