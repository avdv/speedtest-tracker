-- Add download_bytes and upload_bytes columns to results table
-- This migration is safe to run multiple times

-- For SQLite, ALTER TABLE ADD COLUMN will fail if column already exists
-- We'll handle this at the application level by catching the error

ALTER TABLE results ADD COLUMN download_bytes BIGINT;
ALTER TABLE results ADD COLUMN upload_bytes BIGINT;
