use std::sync::Arc;

use tenet_errors::AppError;

use crate::domain::ports::PageFetcher;
use crate::domain::ports::TargetAnalyzer;
use crate::domain::work::{Analysis, ScanClaim};

pub struct AnalyzeMobileTarget {
    fetcher: Arc<dyn PageFetcher>,
}

impl AnalyzeMobileTarget {
    pub fn new(fetcher: Arc<dyn PageFetcher>) -> Self {
        Self { fetcher }
    }
}

#[async_trait::async_trait]
impl TargetAnalyzer for AnalyzeMobileTarget {
    async fn analyze(&self, claim: &ScanClaim) -> Result<Analysis, AppError> {
        if !is_fetchable(&claim.target) {
            return Err(AppError::BadRequest(
                "mobile scans need a url to an apk or a native .so".to_owned(),
            ));
        }
        let bytes = self.fetcher.fetch_bytes(&claim.target).await?;
        let report = tenet_mobile::analyze_binary(&claim.target, &bytes);
        if report.libraries == 0 {
            return Err(AppError::BadRequest(
                "no native library could be read from the target".to_owned(),
            ));
        }
        tracing::info!(
            scan_id = %claim.id,
            libraries = report.libraries,
            findings = report.findings.len(),
            endpoints = report.endpoints.len(),
            "native binary analyzed"
        );
        Ok(Analysis {
            artifacts: Vec::new(),
            findings: report.findings,
            endpoints: report.endpoints,
        })
    }
}

fn is_fetchable(target: &str) -> bool {
    target.starts_with("http://") || target.starts_with("https://")
}
