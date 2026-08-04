use std::time::Duration;

use sqlx::postgres::PgPoolOptions;

pub use sqlx::PgPool;

const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);
const IDLE_TIMEOUT: Duration = Duration::from_secs(300);
const MAX_LIFETIME: Duration = Duration::from_secs(1800);

pub async fn connect(url: &str, max_connections: u32) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(max_connections)
        .min_connections((max_connections / 4).max(1))
        .acquire_timeout(ACQUIRE_TIMEOUT)
        .idle_timeout(Some(IDLE_TIMEOUT))
        .max_lifetime(Some(MAX_LIFETIME))
        .test_before_acquire(false)
        .connect(url)
        .await
}
