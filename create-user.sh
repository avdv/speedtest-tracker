#!/bin/bash

# Simple script to create a test user in the database

if [ -z "$1" ]; then
    echo "Usage: ./create-user.sh <email> [password] [name] [role]"
    echo ""
    echo "Example:"
    echo "  ./create-user.sh admin@example.com admin123"
    echo "  ./create-user.sh admin@example.com admin123 'Admin User' admin"
    echo ""
    echo "Default password: password"
    echo "Default name: Admin"
    echo "Default role: admin"
    exit 1
fi

EMAIL="$1"
PASSWORD="${2:-password}"
NAME="${3:-Admin}"
ROLE="${4:-admin}"

# Hash the password using bcrypt (cost 12)
HASHED=$(cargo run --bin create-user-hash "$PASSWORD" 2>/dev/null)

if [ -z "$HASHED" ]; then
    echo "Error: Could not hash password. Trying alternative method..."
    # Fallback to Python if available
    HASHED=$(python3 -c "import bcrypt; print(bcrypt.hashpw(b'$PASSWORD', bcrypt.gensalt(12)).decode())" 2>/dev/null)
fi

if [ -z "$HASHED" ]; then
    echo "Error: Could not hash password. Install bcrypt:"
    echo "  pip3 install bcrypt"
    exit 1
fi

DATABASE_URL="${DATABASE_URL:-sqlite:./database/database.sqlite}"

echo "Creating user..."
echo "  Email: $EMAIL"
echo "  Password: $PASSWORD"
echo "  Name: $NAME"
echo "  Role: $ROLE"
echo "  Database: $DATABASE_URL"
echo ""

# Create database if using SQLite
if [[ $DATABASE_URL == sqlite:* ]]; then
    DB_FILE="${DATABASE_URL#sqlite:}"
    mkdir -p "$(dirname "$DB_FILE")"
    
    # Create tables if they don't exist
    sqlite3 "$DB_FILE" << EOF
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'user',
    email_verified_at DATETIME,
    remember_token TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

INSERT OR REPLACE INTO users (name, email, password, role, created_at, updated_at)
VALUES ('$NAME', '$EMAIL', '$HASHED', '$ROLE', datetime('now'), datetime('now'));
EOF

    if [ $? -eq 0 ]; then
        echo "✅ User created successfully!"
        echo ""
        echo "Login with:"
        echo "  Email: $EMAIL"
        echo "  Password: $PASSWORD"
    else
        echo "❌ Failed to create user"
        exit 1
    fi
fi

# TODO: Add MySQL and PostgreSQL support
if [[ $DATABASE_URL == mysql:* ]] || [[ $DATABASE_URL == postgresql:* ]]; then
    echo "⚠️  MySQL/PostgreSQL support not yet implemented in this script."
    echo "Please use the database client to insert manually:"
    echo ""
    echo "INSERT INTO users (name, email, password, role, created_at, updated_at)"
    echo "VALUES ('$NAME', '$EMAIL', '$HASHED', '$ROLE', NOW(), NOW());"
fi
