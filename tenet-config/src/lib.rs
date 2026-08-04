mod env;
mod from_env;
mod shutdown;
mod tracing_setup;

pub use shutdown::{shutdown_token, terminate_signal};
pub use tokio_util::sync::CancellationToken;
pub use tracing_setup::init_tracing;

#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
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
    pub browser_enabled: bool,
    pub browser_pool_size: usize,
    pub chrome_bin: String,
    pub chrome_ws_url: String,
    pub browser_headful: bool,
    pub browser_no_sandbox: bool,
    pub browser_viewport_width: u32,
    pub browser_viewport_height: u32,
    pub browser_nav_timeout_seconds: u64,
    pub browser_settle_ms: u64,
    pub browser_settle_jitter_ms: u64,
    pub browser_quiet_ms: u64,
    pub browser_stealth: bool,
    pub browser_region: String,
}
