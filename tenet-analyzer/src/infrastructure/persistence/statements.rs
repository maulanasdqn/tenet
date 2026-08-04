pub const CLAIM_SCANS: &str = "UPDATE scans SET status = 'running', started_at = now(), \
     attempts = attempts + 1 WHERE id IN ( \
       SELECT id FROM scans WHERE status = 'queued' ORDER BY created_at \
       FOR UPDATE SKIP LOCKED LIMIT $1 \
     ) RETURNING id, target, kind, engine, max_scripts, attempts, max_attempts";

pub const COMPLETE_SCAN: &str = "UPDATE scans SET status = 'succeeded', finished_at = now(), \
     error = NULL WHERE id = $1";

pub const REQUEUE_SCAN: &str =
    "UPDATE scans SET status = 'queued', started_at = NULL WHERE id = $1";

pub const FAIL_SCAN: &str =
    "UPDATE scans SET status = 'failed', finished_at = now(), error = $2 WHERE id = $1";

pub const INSERT_ARTIFACT: &str = "INSERT INTO artifacts (id, scan_id, url, kind, sha256, \
     byte_size) VALUES ($1, $2, $3, $4, $5, $6)";

pub const INSERT_FINDING: &str = "INSERT INTO findings (id, scan_id, kind, name, value, severity, \
     confidence, evidence) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
     ON CONFLICT (scan_id, kind, name, value) DO NOTHING";

pub const INSERT_ENDPOINT: &str = "INSERT INTO endpoints (id, scan_id, method, path, base_url, \
     source, confidence) VALUES ($1, $2, $3, $4, $5, $6, $7) \
     ON CONFLICT (scan_id, method, path) DO NOTHING";
