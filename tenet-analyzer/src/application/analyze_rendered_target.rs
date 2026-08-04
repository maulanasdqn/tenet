use std::sync::Arc;

use tenet_errors::AppError;
use tenet_web::PageSnapshot;

use crate::application::challenge::challenge_findings;
use crate::application::harvest::harvest_scripts;
use crate::application::observed::observed_endpoints;
use crate::application::observed_findings::observed_findings;
use crate::application::web_analysis::{analyse, AnalysisInput};
use crate::domain::ports::{PageFetcher, PageRenderer, TargetAnalyzer};
use crate::domain::work::{Analysis, ArtifactRecord, ScanClaim};

pub struct AnalyzeRenderedTarget {
    renderer: Arc<dyn PageRenderer>,
    fetcher: Arc<dyn PageFetcher>,
}

impl AnalyzeRenderedTarget {
    pub fn new(renderer: Arc<dyn PageRenderer>, fetcher: Arc<dyn PageFetcher>) -> Self {
        Self { renderer, fetcher }
    }
}

#[async_trait::async_trait]
impl TargetAnalyzer for AnalyzeRenderedTarget {
    async fn analyze(&self, claim: &ScanClaim) -> Result<Analysis, AppError> {
        let rendered = self.renderer.render(&claim.target).await?;
        let document = &rendered.document;
        let mut artifacts = vec![ArtifactRecord::from_document(document, "rendered")];
        let scripts = harvest_scripts(
            &self.fetcher,
            &document.body,
            &document.url,
            claim.max_scripts.max(0) as usize,
            &mut artifacts,
        )
        .await;

        let page = PageSnapshot::new(
            document.url.clone(),
            document.status,
            document.headers.clone(),
            document.body.clone(),
        );
        let mut findings = observed_findings(&rendered);
        findings.extend(challenge_findings(&rendered.document));
        let analysis = analyse(AnalysisInput {
            page,
            scripts,
            artifacts,
            observed_endpoints: observed_endpoints(&rendered.observed),
            observed_findings: findings,
        });

        tracing::info!(
            scan_id = %claim.id,
            observed = rendered.observed.len(),
            storage_keys = rendered.storage_keys.len(),
            endpoints = analysis.endpoints.len(),
            findings = analysis.findings.len(),
            "rendered target analyzed"
        );
        Ok(analysis)
    }
}
