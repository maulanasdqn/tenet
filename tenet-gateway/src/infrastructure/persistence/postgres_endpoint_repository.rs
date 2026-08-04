use tenet_database::PgPool;
use tenet_errors::AppError;

use crate::domain::artifact::EndpointRecord;
use crate::domain::ports::EndpointRepository;

const SELECT_ENDPOINTS: &str = "SELECT method, path, base_url, source, confidence, created_at \
                                FROM endpoints WHERE scan_id = $1 ORDER BY path, method";

pub struct PostgresEndpointRepository {
    pool: PgPool,
}

impl PostgresEndpointRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl EndpointRepository for PostgresEndpointRepository {
    async fn list_for_scan(&self, scan_id: &str) -> Result<Vec<EndpointRecord>, AppError> {
        let records = sqlx::query_as::<_, EndpointRecord>(SELECT_ENDPOINTS)
            .bind(scan_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(records)
    }
}
