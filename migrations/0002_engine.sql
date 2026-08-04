ALTER TABLE scans ADD COLUMN IF NOT EXISTS engine TEXT NOT NULL DEFAULT 'http';

CREATE INDEX IF NOT EXISTS scans_engine_idx ON scans (engine);
