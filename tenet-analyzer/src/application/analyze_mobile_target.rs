use std::sync::Arc;

use tenet_errors::AppError;
use tenet_mobile::BinaryAnalyzer;

use crate::domain::ports::TargetAnalyzer;
use crate::domain::work::{Analysis, ScanClaim};

pub struct AnalyzeMobileTarget {
    binaries: Arc<dyn BinaryAnalyzer>,
}

impl AnalyzeMobileTarget {
    pub fn new(binaries: Arc<dyn BinaryAnalyzer>) -> Self {
        Self { binaries }
    }
}

#[async_trait::async_trait]
impl TargetAnalyzer for AnalyzeMobileTarget {
    async fn analyze(&self, claim: &ScanClaim) -> Result<Analysis, AppError> {
        let report = self.binaries.analyze(&claim.target).await?;
        Ok(Analysis {
            artifacts: Vec::new(),
            findings: report.findings,
            endpoints: report.endpoints,
        })
    }
}
