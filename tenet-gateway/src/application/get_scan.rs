use std::sync::Arc;

use tenet_errors::AppError;

use crate::domain::ports::ScanRepository;
use crate::domain::scan::ScanRecord;

pub struct GetScan {
    repository: Arc<dyn ScanRepository>,
}

impl GetScan {
    pub fn new(repository: Arc<dyn ScanRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, id: &str) -> Result<ScanRecord, AppError> {
        self.repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("no scan with id {id}")))
    }
}
