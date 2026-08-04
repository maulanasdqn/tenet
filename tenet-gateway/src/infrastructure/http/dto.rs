use chrono::{DateTime, Utc};
use garde::Validate;
use serde::{Deserialize, Serialize};
use tenet_types::TargetKind;
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
    #[garde(inner(range(min = 1, max = MAX_SCRIPTS)))]
    pub max_scripts: Option<i16>,
}

fn default_kind() -> TargetKind {
    TargetKind::Web
}

impl From<CreateScanRequest> for SubmitScanInput {
    fn from(request: CreateScanRequest) -> Self {
        Self {
            target: request.target,
            kind: request.kind,
            max_scripts: request.max_scripts,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ScanCreatedResponse {
    pub scan_id: String,
    pub target: String,
    pub kind: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

impl From<NewScan> for ScanCreatedResponse {
    fn from(scan: NewScan) -> Self {
        Self {
            scan_id: scan.id,
            target: scan.target,
            kind: scan.kind.as_str().to_owned(),
            status: "queued".to_owned(),
            created_at: scan.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use garde::Validate;
    use tenet_types::TargetKind;

    use super::CreateScanRequest;

    fn request(target: &str, max_scripts: Option<i16>) -> CreateScanRequest {
        CreateScanRequest {
            target: target.to_owned(),
            kind: TargetKind::Web,
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
    fn the_kind_defaults_to_web() {
        let parsed: CreateScanRequest = serde_json::from_str(r#"{"target":"https://example.com"}"#)
            .unwrap_or(request("x", None));
        assert_eq!(parsed.kind, TargetKind::Web);
    }
}
