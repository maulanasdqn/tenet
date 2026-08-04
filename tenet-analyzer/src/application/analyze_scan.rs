use std::sync::Arc;

use tenet_errors::AppError;
use tenet_types::{Engine, TargetKind};

use crate::domain::ports::{AnalysisWriter, ScanQueue, TargetAnalyzer};
use crate::domain::work::ScanClaim;

pub struct Analyzers {
    pub web: Arc<dyn TargetAnalyzer>,
    pub rendered: Arc<dyn TargetAnalyzer>,
    pub mobile: Arc<dyn TargetAnalyzer>,
}

pub struct AnalyzeScan {
    queue: Arc<dyn ScanQueue>,
    writer: Arc<dyn AnalysisWriter>,
    analyzers: Analyzers,
}

impl AnalyzeScan {
    pub fn new(
        queue: Arc<dyn ScanQueue>,
        writer: Arc<dyn AnalysisWriter>,
        analyzers: Analyzers,
    ) -> Self {
        Self {
            queue,
            writer,
            analyzers,
        }
    }

    pub async fn execute(&self, claim: ScanClaim) -> Result<(), AppError> {
        match self.analyzer_for(&claim).analyze(&claim).await {
            Ok(analysis) => {
                self.writer.store(&claim.id, &analysis).await?;
                self.queue.complete(&claim.id).await
            }
            Err(err) => self.settle_failure(&claim, &err).await,
        }
    }

    fn analyzer_for(&self, claim: &ScanClaim) -> &Arc<dyn TargetAnalyzer> {
        match TargetKind::from_name(&claim.kind) {
            TargetKind::Mobile => &self.analyzers.mobile,
            TargetKind::Web => match Engine::from_name(&claim.engine) {
                Engine::Browser => &self.analyzers.rendered,
                Engine::Http => &self.analyzers.web,
            },
        }
    }

    async fn settle_failure(&self, claim: &ScanClaim, err: &AppError) -> Result<(), AppError> {
        if is_retryable(claim, err) {
            tracing::warn!(scan_id = %claim.id, error = %err, "scan deferred");
            return self.queue.requeue(&claim.id).await;
        }
        tracing::error!(scan_id = %claim.id, error = %err, "scan failed");
        self.queue.fail(&claim.id, &err.to_string()).await
    }
}

fn is_retryable(claim: &ScanClaim, err: &AppError) -> bool {
    err.is_transient() && claim.attempts < claim.max_attempts
}

#[cfg(test)]
mod tests {
    use tenet_errors::AppError;

    use crate::domain::work::ScanClaim;

    use super::is_retryable;

    fn claim(attempts: i16, max_attempts: i16) -> ScanClaim {
        ScanClaim {
            id: "scan".to_owned(),
            target: "https://example.com".to_owned(),
            kind: "web".to_owned(),
            engine: "http".to_owned(),
            max_scripts: 10,
            attempts,
            max_attempts,
        }
    }

    #[test]
    fn a_transient_failure_with_attempts_left_is_retried() {
        let err = AppError::Unavailable("connection reset".to_owned());
        assert!(is_retryable(&claim(1, 3), &err));
    }

    #[test]
    fn the_last_attempt_is_never_retried() {
        let err = AppError::Unavailable("connection reset".to_owned());
        assert!(!is_retryable(&claim(3, 3), &err));
    }

    #[test]
    fn a_permanent_failure_is_never_retried() {
        let err = AppError::ValidationError("target is not a valid url".to_owned());
        assert!(!is_retryable(&claim(1, 3), &err));
    }
}
