use std::sync::Arc;

use tenet_errors::AppError;

use crate::domain::artifact::FindingRecord;
use crate::domain::ports::{FindingRepository, ScanRepository};

pub struct ListFindings {
    scans: Arc<dyn ScanRepository>,
    findings: Arc<dyn FindingRepository>,
}

impl ListFindings {
    pub fn new(scans: Arc<dyn ScanRepository>, findings: Arc<dyn FindingRepository>) -> Self {
        Self { scans, findings }
    }

    pub async fn execute(&self, scan_id: &str) -> Result<Vec<FindingRecord>, AppError> {
        if self.scans.find_by_id(scan_id).await?.is_none() {
            return Err(AppError::NotFound(format!("no scan with id {scan_id}")));
        }
        self.findings.list_for_scan(scan_id).await
    }
}
