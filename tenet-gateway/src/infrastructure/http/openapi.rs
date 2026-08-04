use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa::{Modify, OpenApi};

use crate::infrastructure::http::dto::{CreateScanRequest, ScanCreatedResponse};
use crate::infrastructure::http::views::{EndpointResponse, FindingResponse, ScanResponse};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::infrastructure::http::handlers::health,
        crate::infrastructure::http::handlers::create_scan,
        crate::infrastructure::http::handlers::get_scan,
        crate::infrastructure::http::handlers::list_findings,
        crate::infrastructure::http::handlers::list_endpoints,
        crate::infrastructure::http::handlers::export_openapi,
    ),
    components(schemas(
        CreateScanRequest,
        ScanCreatedResponse,
        ScanResponse,
        FindingResponse,
        EndpointResponse
    )),
    modifiers(&ApiKeySecurity),
    tags(
        (name = "health", description = "liveness"),
        (name = "scans", description = "reverse engineering scans")
    )
)]
pub struct ApiDoc;

struct ApiKeySecurity;

impl Modify for ApiKeySecurity {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let Some(components) = openapi.components.as_mut() else {
            return;
        };
        components.add_security_scheme(
            "api_key",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("x-api-key"))),
        );
    }
}
