use crate::env::{env_or, env_parse};
use crate::Config;

impl Config {
    pub fn from_env() -> Result<Self, String> {
        Ok(Self {
            database_url: env_or(
                "DATABASE_URL",
                "postgres://tenet:tenet@localhost:5432/tenet",
            ),
            db_max_connections: env_parse("DB_MAX_CONNECTIONS", "10")?,
            api_key: env_or("API_KEY", "dev-key"),
            http_addr: env_or("HTTP_ADDR", "0.0.0.0:8080"),
            analyzer_batch_size: env_parse("ANALYZER_BATCH_SIZE", "4")?,
            analyzer_interval_ms: env_parse("ANALYZER_INTERVAL_MS", "1000")?,
            analyzer_concurrency: env_parse("ANALYZER_CONCURRENCY", "4")?,
            scan_max_attempts: env_parse("SCAN_MAX_ATTEMPTS", "3")?,
            scan_max_scripts: env_parse("SCAN_MAX_SCRIPTS", "25")?,
            scan_max_body_bytes: env_parse("SCAN_MAX_BODY_BYTES", "8388608")?,
            fetch_timeout_seconds: env_parse("FETCH_TIMEOUT_SECONDS", "20")?,
            fetch_connect_timeout_seconds: env_parse("FETCH_CONNECT_TIMEOUT_SECONDS", "5")?,
            fetch_max_redirects: env_parse("FETCH_MAX_REDIRECTS", "5")?,
            fetch_user_agent: env_or("FETCH_USER_AGENT", "tenet/0.1"),
        })
    }
}
