use axum::middleware;
use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::infrastructure::http::auth::require_api_key;
use crate::infrastructure::http::handlers;
use crate::infrastructure::http::openapi::ApiDoc;
use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    let protected = Router::new()
        .route("/v1/scans", post(handlers::create_scan))
        .route("/v1/scans/{id}", get(handlers::get_scan))
        .route("/v1/scans/{id}/findings", get(handlers::list_findings))
        .route("/v1/scans/{id}/endpoints", get(handlers::list_endpoints))
        .route("/v1/scans/{id}/openapi", get(handlers::export_openapi))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            require_api_key,
        ));

    Router::new()
        .route("/healthz", get(handlers::health))
        .merge(SwaggerUi::new("/docs").url("/openapi/gateway.json", ApiDoc::openapi()))
        .merge(protected)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
