CREATE TABLE IF NOT EXISTS scans (
    id            TEXT PRIMARY KEY,
    target        TEXT        NOT NULL,
    kind          TEXT        NOT NULL,
    status        TEXT        NOT NULL DEFAULT 'queued',
    max_scripts   SMALLINT    NOT NULL DEFAULT 25,
    attempts      SMALLINT    NOT NULL DEFAULT 0,
    max_attempts  SMALLINT    NOT NULL DEFAULT 3,
    error         TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    started_at    TIMESTAMPTZ,
    finished_at   TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS scans_claim_idx ON scans (status, created_at);

CREATE TABLE IF NOT EXISTS artifacts (
    id         TEXT PRIMARY KEY,
    scan_id    TEXT        NOT NULL REFERENCES scans (id) ON DELETE CASCADE,
    url        TEXT        NOT NULL,
    kind       TEXT        NOT NULL,
    sha256     TEXT        NOT NULL,
    byte_size  BIGINT      NOT NULL,
    fetched_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS artifacts_scan_idx ON artifacts (scan_id);

CREATE TABLE IF NOT EXISTS findings (
    id         TEXT PRIMARY KEY,
    scan_id    TEXT        NOT NULL REFERENCES scans (id) ON DELETE CASCADE,
    kind       TEXT        NOT NULL,
    name       TEXT        NOT NULL,
    value      TEXT,
    severity   TEXT        NOT NULL DEFAULT 'info',
    confidence REAL        NOT NULL DEFAULT 0.5,
    evidence   TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (scan_id, kind, name, value)
);

CREATE INDEX IF NOT EXISTS findings_scan_idx ON findings (scan_id, kind);

CREATE TABLE IF NOT EXISTS endpoints (
    id         TEXT PRIMARY KEY,
    scan_id    TEXT        NOT NULL REFERENCES scans (id) ON DELETE CASCADE,
    method     TEXT        NOT NULL,
    path       TEXT        NOT NULL,
    base_url   TEXT,
    source     TEXT        NOT NULL,
    confidence REAL        NOT NULL DEFAULT 0.5,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (scan_id, method, path)
);

CREATE INDEX IF NOT EXISTS endpoints_scan_idx ON endpoints (scan_id);
