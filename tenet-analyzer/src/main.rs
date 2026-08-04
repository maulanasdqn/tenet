mod application;
mod domain;
mod infrastructure;

use std::sync::Arc;
use std::time::Duration;

use application::analyze_mobile_target::AnalyzeMobileTarget;
use application::analyze_scan::AnalyzeScan;
use application::analyze_web_target::AnalyzeWebTarget;
use application::run_batch::RunBatch;
use domain::ports::{PageFetcher, TargetAnalyzer};
use infrastructure::fetch::reqwest_page_fetcher::{FetchSettings, ReqwestPageFetcher};
use infrastructure::persistence::postgres_analysis_writer::PostgresAnalysisWriter;
use infrastructure::persistence::postgres_scan_queue::PostgresScanQueue;
use infrastructure::scan_loop::run_scan_loop;
use tenet_mobile::PendingBinaryAnalyzer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tenet_config::init_tracing();
    let config = tenet_config::Config::from_env()?;

    let pool = tenet_database::connect(&config.database_url, config.db_max_connections).await?;

    let fetcher: Arc<dyn PageFetcher> = Arc::new(ReqwestPageFetcher::new(FetchSettings {
        timeout_seconds: config.fetch_timeout_seconds,
        connect_timeout_seconds: config.fetch_connect_timeout_seconds,
        max_redirects: config.fetch_max_redirects,
        user_agent: config.fetch_user_agent.clone(),
        max_body_bytes: config.scan_max_body_bytes,
    })?);

    let web: Arc<dyn TargetAnalyzer> = Arc::new(AnalyzeWebTarget::new(fetcher));
    let mobile: Arc<dyn TargetAnalyzer> =
        Arc::new(AnalyzeMobileTarget::new(Arc::new(PendingBinaryAnalyzer)));
    let queue = Arc::new(PostgresScanQueue::new(pool.clone()));
    let writer = Arc::new(PostgresAnalysisWriter::new(pool));

    let run = Arc::new(RunBatch::new(
        queue.clone(),
        Arc::new(AnalyzeScan::new(queue, writer, web, mobile)),
        config.analyzer_batch_size,
        config.analyzer_concurrency,
    ));

    let shutdown = tenet_config::shutdown_token();
    tracing::info!(
        batch = config.analyzer_batch_size,
        concurrency = config.analyzer_concurrency,
        "tenet-analyzer polling for scans"
    );
    run_scan_loop(
        run,
        Duration::from_millis(config.analyzer_interval_ms),
        shutdown,
    )
    .await;
    Ok(())
}
