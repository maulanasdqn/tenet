use tenet_database::PgPool;
use tenet_errors::AppError;

use crate::domain::ports::ScanQueue;
use crate::domain::work::ScanClaim;
use crate::infrastructure::persistence::statements::{
    CLAIM_SCANS, COMPLETE_SCAN, FAIL_SCAN, REQUEUE_SCAN,
};

pub struct PostgresScanQueue {
    pool: PgPool,
}

impl PostgresScanQueue {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn settle(&self, statement: &str, scan_id: &str) -> Result<(), AppError> {
        sqlx::query(statement)
            .bind(scan_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl ScanQueue for PostgresScanQueue {
    async fn claim(&self, limit: i64) -> Result<Vec<ScanClaim>, AppError> {
        let claims = sqlx::query_as::<_, ScanClaim>(CLAIM_SCANS)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;
        Ok(claims)
    }

    async fn complete(&self, scan_id: &str) -> Result<(), AppError> {
        self.settle(COMPLETE_SCAN, scan_id).await
    }

    async fn requeue(&self, scan_id: &str) -> Result<(), AppError> {
        self.settle(REQUEUE_SCAN, scan_id).await
    }

    async fn fail(&self, scan_id: &str, error: &str) -> Result<(), AppError> {
        sqlx::query(FAIL_SCAN)
            .bind(scan_id)
            .bind(error)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
