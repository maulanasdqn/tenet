mod application;
mod domain;
mod infrastructure;
mod state;

use std::sync::Arc;

use application::export_openapi::ExportOpenapi;
use application::get_scan::GetScan;
use application::list_endpoints::ListEndpoints;
use application::list_findings::ListFindings;
use application::submit_scan::SubmitScan;
use infrastructure::http::routes;
use infrastructure::persistence::postgres_endpoint_repository::PostgresEndpointRepository;
use infrastructure::persistence::postgres_finding_repository::PostgresFindingRepository;
use infrastructure::persistence::postgres_scan_repository::PostgresScanRepository;
use state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tenet_config::init_tracing();
    let config = tenet_config::Config::from_env()?;

    let pool = tenet_database::connect(&config.database_url, config.db_max_connections).await?;
    sqlx::migrate!("../migrations").run(&pool).await?;

    let scans = Arc::new(PostgresScanRepository::new(pool.clone()));
    let findings = Arc::new(PostgresFindingRepository::new(pool.clone()));
    let endpoints = Arc::new(PostgresEndpointRepository::new(pool));

    let state = AppState {
        submit_scan: Arc::new(SubmitScan::new(
            scans.clone(),
            config.scan_max_scripts,
            config.scan_max_attempts,
        )),
        get_scan: Arc::new(GetScan::new(scans.clone())),
        list_findings: Arc::new(ListFindings::new(scans.clone(), findings.clone())),
        list_endpoints: Arc::new(ListEndpoints::new(scans.clone(), endpoints.clone())),
        export_openapi: Arc::new(ExportOpenapi::new(scans, endpoints, findings)),
        api_key: Arc::new(config.api_key),
    };

    let listener = tokio::net::TcpListener::bind(&config.http_addr).await?;
    tracing::info!(addr = %config.http_addr, "tenet-gateway listening");
    axum::serve(listener, routes::router(state))
        .with_graceful_shutdown(tenet_config::terminate_signal())
        .await?;
    Ok(())
}
