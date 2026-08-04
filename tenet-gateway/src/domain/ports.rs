use tenet_errors::AppError;

use crate::domain::artifact::{EndpointRecord, FindingRecord};
use crate::domain::scan::{NewScan, ScanRecord};

#[async_trait::async_trait]
pub trait ScanRepository: Send + Sync {
    async fn insert(&self, scan: &NewScan) -> Result<(), AppError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<ScanRecord>, AppError>;
}

#[async_trait::async_trait]
pub trait FindingRepository: Send + Sync {
    async fn list_for_scan(&self, scan_id: &str) -> Result<Vec<FindingRecord>, AppError>;
}

#[async_trait::async_trait]
pub trait EndpointRepository: Send + Sync {
    async fn list_for_scan(&self, scan_id: &str) -> Result<Vec<EndpointRecord>, AppError>;
}
