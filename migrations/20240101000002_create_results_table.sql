-- Create results table
CREATE TABLE IF NOT EXISTS results (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ping REAL,
    download INTEGER NOT NULL,
    upload INTEGER NOT NULL,
    server_id INTEGER,
    server_host TEXT,
    server_name TEXT,
    server_location TEXT,
    server_country TEXT,
    url TEXT,
    scheduled INTEGER NOT NULL DEFAULT 0,
    service TEXT NOT NULL DEFAULT 'ookla',
    status TEXT NOT NULL DEFAULT 'completed',
    data TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
