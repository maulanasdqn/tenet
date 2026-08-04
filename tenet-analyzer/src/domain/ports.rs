use tenet_errors::AppError;

use crate::domain::work::{Analysis, FetchedDocument, RenderedDocument, ScanClaim};

#[async_trait::async_trait]
pub trait ScanQueue: Send + Sync {
    async fn claim(&self, limit: i64) -> Result<Vec<ScanClaim>, AppError>;
    async fn complete(&self, scan_id: &str) -> Result<(), AppError>;
    async fn requeue(&self, scan_id: &str) -> Result<(), AppError>;
    async fn fail(&self, scan_id: &str, error: &str) -> Result<(), AppError>;
}

#[async_trait::async_trait]
pub trait AnalysisWriter: Send + Sync {
    async fn store(&self, scan_id: &str, analysis: &Analysis) -> Result<(), AppError>;
}

#[async_trait::async_trait]
pub trait TargetAnalyzer: Send + Sync {
    async fn analyze(&self, claim: &ScanClaim) -> Result<Analysis, AppError>;
}

#[async_trait::async_trait]
pub trait PageFetcher: Send + Sync {
    async fn fetch(&self, url: &str) -> Result<FetchedDocument, AppError>;
}

#[async_trait::async_trait]
pub trait PageRenderer: Send + Sync {
    async fn render(&self, url: &str) -> Result<RenderedDocument, AppError>;
}
