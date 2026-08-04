use std::sync::Arc;

use futures_util::stream::{self, StreamExt};
use tenet_errors::AppError;

use crate::application::analyze_scan::AnalyzeScan;
use crate::domain::ports::ScanQueue;
use crate::domain::work::ScanClaim;

pub struct RunBatch {
    queue: Arc<dyn ScanQueue>,
    analyze: Arc<AnalyzeScan>,
    batch_size: i64,
    concurrency: usize,
}

impl RunBatch {
    pub fn new(
        queue: Arc<dyn ScanQueue>,
        analyze: Arc<AnalyzeScan>,
        batch_size: i64,
        concurrency: usize,
    ) -> Self {
        Self {
            queue,
            analyze,
            batch_size,
            concurrency: concurrency.max(1),
        }
    }

    pub async fn execute(&self) -> Result<usize, AppError> {
        let claims = self.queue.claim(self.batch_size).await?;
        if claims.is_empty() {
            return Ok(0);
        }

        let claimed = claims.len();
        stream::iter(claims)
            .for_each_concurrent(self.concurrency, |claim| {
                settle(self.analyze.clone(), claim)
            })
            .await;
        Ok(claimed)
    }
}

async fn settle(analyze: Arc<AnalyzeScan>, claim: ScanClaim) {
    let scan_id = claim.id.clone();
    if let Err(err) = analyze.execute(claim).await {
        tracing::error!(scan_id = %scan_id, error = %err, "scan could not be settled");
    }
}
