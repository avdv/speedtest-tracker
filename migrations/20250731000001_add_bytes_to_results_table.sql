-- Add download_bytes and upload_bytes columns to results table
ALTER TABLE results ADD COLUMN download_bytes BIGINT;
ALTER TABLE results ADD COLUMN upload_bytes BIGINT;
