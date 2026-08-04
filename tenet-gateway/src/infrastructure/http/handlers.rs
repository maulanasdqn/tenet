use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde_json::Value;
use tenet_errors::AppError;
use tenet_types::{ListResponse, SingleResponse};

use crate::infrastructure::http::dto::{CreateScanRequest, ScanCreatedResponse};
use crate::infrastructure::http::validation::ValidatedJson;
use crate::infrastructure::http::views::{EndpointResponse, FindingResponse, ScanResponse};
use crate::state::AppState;

#[utoipa::path(
    get, path = "/healthz", tag = "health",
    responses((status = 200, description = "the service is up"))
)]
pub async fn health() -> StatusCode {
    StatusCode::OK
}

#[utoipa::path(
    post, path = "/v1/scans", tag = "scans",
    request_body = CreateScanRequest,
    security(("api_key" = [])),
    responses(
        (status = 202, description = "scan queued", body = SingleResponse<ScanCreatedResponse>),
        (status = 401, description = "the api key was missing or rejected"),
        (status = 422, description = "the request failed validation")
    )
)]
pub async fn create_scan(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<CreateScanRequest>,
) -> Result<(StatusCode, Json<SingleResponse<ScanCreatedResponse>>), AppError> {
    let scan = state.submit_scan.execute(payload.into()).await?;
    Ok((
        StatusCode::ACCEPTED,
        Json(SingleResponse::new(scan.into(), "scan accepted")),
    ))
}

#[utoipa::path(
    get, path = "/v1/scans/{id}", tag = "scans",
    params(("id" = String, Path, description = "scan id")),
    security(("api_key" = [])),
    responses(
        (status = 200, description = "the scan", body = SingleResponse<ScanResponse>),
        (status = 404, description = "no such scan")
    )
)]
pub async fn get_scan(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SingleResponse<ScanResponse>>, AppError> {
    let scan = state.get_scan.execute(&id).await?;
    Ok(Json(SingleResponse::new(scan.into(), "scan found")))
}

#[utoipa::path(
    get, path = "/v1/scans/{id}/findings", tag = "scans",
    params(("id" = String, Path, description = "scan id")),
    security(("api_key" = [])),
    responses(
        (status = 200, description = "what the scan learned", body = ListResponse<FindingResponse>),
        (status = 404, description = "no such scan")
    )
)]
pub async fn list_findings(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ListResponse<FindingResponse>>, AppError> {
    let findings = state.list_findings.execute(&id).await?;
    Ok(Json(ListResponse::new(
        findings.into_iter().map(Into::into).collect(),
    )))
}

#[utoipa::path(
    get, path = "/v1/scans/{id}/endpoints", tag = "scans",
    params(("id" = String, Path, description = "scan id")),
    security(("api_key" = [])),
    responses(
        (status = 200, description = "the endpoints found", body = ListResponse<EndpointResponse>),
        (status = 404, description = "no such scan")
    )
)]
pub async fn list_endpoints(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ListResponse<EndpointResponse>>, AppError> {
    let endpoints = state.list_endpoints.execute(&id).await?;
    Ok(Json(ListResponse::new(
        endpoints.into_iter().map(Into::into).collect(),
    )))
}

#[utoipa::path(
    get, path = "/v1/scans/{id}/openapi", tag = "scans",
    params(("id" = String, Path, description = "scan id")),
    security(("api_key" = [])),
    responses(
        (status = 200, description = "an openapi document for the discovered endpoints"),
        (status = 404, description = "no such scan")
    )
)]
pub async fn export_openapi(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, AppError> {
    let document = state.export_openapi.execute(&id).await?;
    Ok(Json(document))
}
