use std::sync::Arc;

use tenet_errors::AppError;

use crate::domain::artifact::EndpointRecord;
use crate::domain::ports::{EndpointRepository, ScanRepository};

pub struct ListEndpoints {
    scans: Arc<dyn ScanRepository>,
    endpoints: Arc<dyn EndpointRepository>,
}

impl ListEndpoints {
    pub fn new(scans: Arc<dyn ScanRepository>, endpoints: Arc<dyn EndpointRepository>) -> Self {
        Self { scans, endpoints }
    }

    pub async fn execute(&self, scan_id: &str) -> Result<Vec<EndpointRecord>, AppError> {
        if self.scans.find_by_id(scan_id).await?.is_none() {
            return Err(AppError::NotFound(format!("no scan with id {scan_id}")));
        }
        self.endpoints.list_for_scan(scan_id).await
    }
}
