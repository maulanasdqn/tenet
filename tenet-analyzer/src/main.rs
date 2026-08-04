mod application;
mod domain;
mod infrastructure;

use std::sync::Arc;
use std::time::Duration;

use application::analyze_mobile_target::AnalyzeMobileTarget;
use application::analyze_rendered_target::AnalyzeRenderedTarget;
use application::analyze_scan::{AnalyzeScan, Analyzers};
use application::analyze_web_target::AnalyzeWebTarget;
use application::run_batch::RunBatch;
use application::unavailable_target::UnavailableTarget;
use domain::ports::{PageFetcher, PageRenderer, TargetAnalyzer};
use infrastructure::browser::chromium_page_renderer::ChromiumPageRenderer;
use infrastructure::fetch::reqwest_page_fetcher::{FetchSettings, ReqwestPageFetcher};
use infrastructure::persistence::postgres_analysis_writer::PostgresAnalysisWriter;
use infrastructure::persistence::postgres_scan_queue::PostgresScanQueue;
use infrastructure::scan_loop::run_scan_loop;
use tenet_browser::RenderSettings;
use tenet_config::Config;
use tenet_errors::AppError;
use tenet_mobile::PendingBinaryAnalyzer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tenet_config::init_tracing();
    let config = Config::from_env()?;

    let pool = tenet_database::connect(&config.database_url, config.db_max_connections).await?;

    let fetcher: Arc<dyn PageFetcher> = Arc::new(ReqwestPageFetcher::new(FetchSettings {
        timeout_seconds: config.fetch_timeout_seconds,
        connect_timeout_seconds: config.fetch_connect_timeout_seconds,
        max_redirects: config.fetch_max_redirects,
        user_agent: config.fetch_user_agent.clone(),
        max_body_bytes: config.scan_max_body_bytes,
    })?);

    let analyzers = Analyzers {
        web: Arc::new(AnalyzeWebTarget::new(fetcher.clone())),
        rendered: build_rendered(&config, &fetcher).await,
        mobile: Arc::new(AnalyzeMobileTarget::new(Arc::new(PendingBinaryAnalyzer))),
    };

    let queue = Arc::new(PostgresScanQueue::new(pool.clone()));
    let writer = Arc::new(PostgresAnalysisWriter::new(pool));
    let run = Arc::new(RunBatch::new(
        queue.clone(),
        Arc::new(AnalyzeScan::new(queue, writer, analyzers)),
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

async fn build_rendered(
    config: &Config,
    fetcher: &Arc<dyn PageFetcher>,
) -> Arc<dyn TargetAnalyzer> {
    if !config.browser_enabled {
        tracing::info!("the browser engine is disabled");
        return Arc::new(UnavailableTarget::new(
            "the browser engine is disabled on this analyzer",
        ));
    }

    match open_renderer(config).await {
        Ok(renderer) => Arc::new(AnalyzeRenderedTarget::new(renderer, fetcher.clone())),
        Err(err) => {
            tracing::warn!(error = %err, "chromium is unavailable, browser scans will fail");
            Arc::new(UnavailableTarget::new(format!(
                "the browser engine could not start: {err}"
            )))
        }
    }
}

async fn open_renderer(config: &Config) -> Result<Arc<dyn PageRenderer>, AppError> {
    let renderer =
        ChromiumPageRenderer::open(render_settings(config), config.browser_pool_size).await?;
    tracing::info!(pool_size = config.browser_pool_size, "browser engine ready");
    Ok(Arc::new(renderer))
}

fn render_settings(config: &Config) -> RenderSettings {
    RenderSettings {
        chrome_bin: config.chrome_bin.clone(),
        chrome_ws_url: config.chrome_ws_url.clone(),
        headful: config.browser_headful,
        no_sandbox: config.browser_no_sandbox,
        viewport_width: config.browser_viewport_width,
        viewport_height: config.browser_viewport_height,
        nav_timeout_seconds: config.browser_nav_timeout_seconds,
        settle_ms: config.browser_settle_ms,
        settle_jitter_ms: config.browser_settle_jitter_ms,
        quiet_ms: config.browser_quiet_ms,
        stealth: config.browser_stealth,
        region: config.browser_region.clone(),
    }
}
