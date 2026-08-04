use std::sync::Arc;
use std::time::Duration;

use tenet_config::CancellationToken;

use crate::application::run_batch::RunBatch;

pub async fn run_scan_loop(run: Arc<RunBatch>, interval: Duration, shutdown: CancellationToken) {
    while tick(interval, &shutdown).await {
        settle_batch(&run).await;
    }
    tracing::info!("scan loop stopped");
}

async fn tick(interval: Duration, shutdown: &CancellationToken) -> bool {
    tokio::select! {
        () = shutdown.cancelled() => false,
        () = tokio::time::sleep(interval) => true,
    }
}

async fn settle_batch(run: &RunBatch) {
    match run.execute().await {
        Ok(0) => {}
        Ok(claimed) => tracing::info!(claimed, "scans settled"),
        Err(err) => tracing::error!(error = %err, "scan batch failed"),
    }
}
