mod env;
mod from_env;
mod shutdown;
mod tracing_setup;

pub use shutdown::{shutdown_token, terminate_signal};
pub use tokio_util::sync::CancellationToken;
pub use tracing_setup::init_tracing;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub db_max_connections: u32,
    pub api_key: String,
    pub http_addr: String,
    pub analyzer_batch_size: i64,
    pub analyzer_interval_ms: u64,
    pub analyzer_concurrency: usize,
    pub scan_max_attempts: i16,
    pub scan_max_scripts: i16,
    pub scan_max_body_bytes: usize,
    pub fetch_timeout_seconds: u64,
    pub fetch_connect_timeout_seconds: u64,
    pub fetch_max_redirects: usize,
    pub fetch_user_agent: String,
}
