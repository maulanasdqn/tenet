use sqlx::{Postgres, Transaction};
use tenet_database::PgPool;
use tenet_errors::AppError;
use tenet_types::{Endpoint, Finding};
use ulid::Ulid;

use crate::domain::ports::AnalysisWriter;
use crate::domain::work::{Analysis, ArtifactRecord};
use crate::infrastructure::persistence::statements::{
    INSERT_ARTIFACT, INSERT_ENDPOINT, INSERT_FINDING,
};

pub struct PostgresAnalysisWriter {
    pool: PgPool,
}

impl PostgresAnalysisWriter {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl AnalysisWriter for PostgresAnalysisWriter {
    async fn store(&self, scan_id: &str, analysis: &Analysis) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await?;
        for artifact in &analysis.artifacts {
            insert_artifact(&mut tx, scan_id, artifact).await?;
        }
        for finding in &analysis.findings {
            insert_finding(&mut tx, scan_id, finding).await?;
        }
        for endpoint in &analysis.endpoints {
            insert_endpoint(&mut tx, scan_id, endpoint).await?;
        }
        tx.commit().await?;
        Ok(())
    }
}

async fn insert_artifact(
    tx: &mut Transaction<'_, Postgres>,
    scan_id: &str,
    artifact: &ArtifactRecord,
) -> Result<(), AppError> {
    sqlx::query(INSERT_ARTIFACT)
        .bind(Ulid::new().to_string())
        .bind(scan_id)
        .bind(&artifact.url)
        .bind(&artifact.kind)
        .bind(&artifact.sha256)
        .bind(artifact.byte_size)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

async fn insert_finding(
    tx: &mut Transaction<'_, Postgres>,
    scan_id: &str,
    finding: &Finding,
) -> Result<(), AppError> {
    sqlx::query(INSERT_FINDING)
        .bind(Ulid::new().to_string())
        .bind(scan_id)
        .bind(finding.kind.as_str())
        .bind(&finding.name)
        .bind(&finding.value)
        .bind(finding.severity.as_str())
        .bind(finding.confidence)
        .bind(&finding.evidence)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

async fn insert_endpoint(
    tx: &mut Transaction<'_, Postgres>,
    scan_id: &str,
    endpoint: &Endpoint,
) -> Result<(), AppError> {
    sqlx::query(INSERT_ENDPOINT)
        .bind(Ulid::new().to_string())
        .bind(scan_id)
        .bind(endpoint.method.as_str())
        .bind(&endpoint.path)
        .bind(&endpoint.base_url)
        .bind(&endpoint.source)
        .bind(endpoint.confidence)
        .execute(&mut **tx)
        .await?;
    Ok(())
}
