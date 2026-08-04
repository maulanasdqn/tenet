use std::sync::Arc;

use crate::application::export_openapi::ExportOpenapi;
use crate::application::get_scan::GetScan;
use crate::application::list_endpoints::ListEndpoints;
use crate::application::list_findings::ListFindings;
use crate::application::submit_scan::SubmitScan;

#[derive(Clone)]
pub struct AppState {
    pub submit_scan: Arc<SubmitScan>,
    pub get_scan: Arc<GetScan>,
    pub list_findings: Arc<ListFindings>,
    pub list_endpoints: Arc<ListEndpoints>,
    pub export_openapi: Arc<ExportOpenapi>,
    pub api_key: Arc<String>,
}
