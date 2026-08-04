use tenet_database::PgPool;
use tenet_errors::AppError;

use crate::domain::artifact::FindingRecord;
use crate::domain::ports::FindingRepository;

const SELECT_FINDINGS: &str = "SELECT kind, name, value, severity, confidence, evidence, \
                               created_at FROM findings WHERE scan_id = $1 \
                               ORDER BY kind, confidence DESC, name";

pub struct PostgresFindingRepository {
    pool: PgPool,
}

impl PostgresFindingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl FindingRepository for PostgresFindingRepository {
    async fn list_for_scan(&self, scan_id: &str) -> Result<Vec<FindingRecord>, AppError> {
        let records = sqlx::query_as::<_, FindingRecord>(SELECT_FINDINGS)
            .bind(scan_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(records)
    }
}
