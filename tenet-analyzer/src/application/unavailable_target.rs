use tenet_errors::AppError;

use crate::domain::ports::TargetAnalyzer;
use crate::domain::work::{Analysis, ScanClaim};

pub struct UnavailableTarget {
    reason: String,
}

impl UnavailableTarget {
    pub fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}

#[async_trait::async_trait]
impl TargetAnalyzer for UnavailableTarget {
    async fn analyze(&self, _claim: &ScanClaim) -> Result<Analysis, AppError> {
        Err(AppError::Unavailable(self.reason.clone()))
    }
}
