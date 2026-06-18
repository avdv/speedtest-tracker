-- Create results table
CREATE TABLE IF NOT EXISTS results (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    service TEXT NOT NULL DEFAULT 'ookla',
    ping REAL,
    download INTEGER,
    upload INTEGER,
    comments TEXT,
    data TEXT,
    status TEXT NOT NULL DEFAULT 'completed',
    scheduled INTEGER NOT NULL DEFAULT FALSE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
