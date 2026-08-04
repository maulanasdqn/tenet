use tenet_database::PgPool;
use tenet_errors::AppError;

use crate::domain::ports::ScanRepository;
use crate::domain::scan::{NewScan, ScanRecord};

const SELECT_SCAN: &str = "SELECT id, target, kind, status, attempts, max_attempts, error, \
                           created_at, started_at, finished_at FROM scans WHERE id = $1";

pub struct PostgresScanRepository {
    pool: PgPool,
}

impl PostgresScanRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ScanRepository for PostgresScanRepository {
    async fn insert(&self, scan: &NewScan) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO scans (id, target, kind, max_scripts, max_attempts, created_at)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(&scan.id)
        .bind(&scan.target)
        .bind(scan.kind.as_str())
        .bind(scan.max_scripts)
        .bind(scan.max_attempts)
        .bind(scan.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<ScanRecord>, AppError> {
        let record = sqlx::query_as::<_, ScanRecord>(SELECT_SCAN)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(record)
    }
}
