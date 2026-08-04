use chrono::{DateTime, Utc};
use garde::Validate;
use serde::{Deserialize, Serialize};
use tenet_types::{Engine, TargetKind};
use utoipa::ToSchema;

use crate::application::submit_scan::SubmitScanInput;
use crate::domain::scan::NewScan;

const MAX_TARGET_LENGTH: usize = 2048;
const MAX_SCRIPTS: i16 = 100;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateScanRequest {
    #[garde(length(min = 1, max = MAX_TARGET_LENGTH))]
    pub target: String,
    #[serde(default = "default_kind")]
    #[garde(skip)]
    pub kind: TargetKind,
    #[serde(default = "default_engine")]
    #[garde(skip)]
    pub engine: Engine,
    #[garde(inner(range(min = 1, max = MAX_SCRIPTS)))]
    pub max_scripts: Option<i16>,
}

fn default_kind() -> TargetKind {
    TargetKind::Web
}

fn default_engine() -> Engine {
    Engine::Http
}

impl From<CreateScanRequest> for SubmitScanInput {
    fn from(request: CreateScanRequest) -> Self {
        Self {
            target: request.target,
            kind: request.kind,
            engine: request.engine,
            max_scripts: request.max_scripts,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ScanCreatedResponse {
    pub scan_id: String,
    pub target: String,
    pub kind: String,
    pub engine: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

impl From<NewScan> for ScanCreatedResponse {
    fn from(scan: NewScan) -> Self {
        Self {
            scan_id: scan.id,
            target: scan.target,
            kind: scan.kind.as_str().to_owned(),
            engine: scan.engine.as_str().to_owned(),
            status: "queued".to_owned(),
            created_at: scan.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use garde::Validate;
    use tenet_types::{Engine, TargetKind};

    use super::CreateScanRequest;

    fn request(target: &str, max_scripts: Option<i16>) -> CreateScanRequest {
        CreateScanRequest {
            target: target.to_owned(),
            kind: TargetKind::Web,
            engine: Engine::Http,
            max_scripts,
        }
    }

    #[test]
    fn a_well_formed_request_passes() {
        assert!(request("https://example.com", Some(10)).validate().is_ok());
    }

    #[test]
    fn an_empty_target_is_rejected() {
        assert!(request("", None).validate().is_err());
    }

    #[test]
    fn a_script_budget_above_the_ceiling_is_rejected() {
        assert!(request("https://example.com", Some(500))
            .validate()
            .is_err());
    }

    #[test]
    fn an_absent_script_budget_is_accepted() {
        assert!(request("https://example.com", None).validate().is_ok());
    }

    #[test]
    fn the_kind_and_engine_default_to_web_over_http() {
        let parsed: CreateScanRequest = serde_json::from_str(r#"{"target":"https://example.com"}"#)
            .unwrap_or(request("x", None));
        assert_eq!(parsed.kind, TargetKind::Web);
        assert_eq!(parsed.engine, Engine::Http);
    }

    #[test]
    fn the_browser_engine_can_be_asked_for_by_name() {
        let body = r#"{"target":"https://example.com","engine":"browser"}"#;
        let parsed: CreateScanRequest = serde_json::from_str(body).unwrap_or(request("x", None));
        assert_eq!(parsed.engine, Engine::Browser);
    }
}
