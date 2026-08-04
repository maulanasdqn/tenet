use std::sync::Arc;

use serde_json::Value;
use tenet_errors::AppError;
use tenet_spec::SpecInput;
use tenet_types::{AuthScheme, Endpoint, HttpMethod};

use crate::domain::artifact::{EndpointRecord, FindingRecord};
use crate::domain::ports::{EndpointRepository, FindingRepository, ScanRepository};

const SPEC_VERSION: &str = "0.1.0";

pub struct ExportOpenapi {
    scans: Arc<dyn ScanRepository>,
    endpoints: Arc<dyn EndpointRepository>,
    findings: Arc<dyn FindingRepository>,
}

impl ExportOpenapi {
    pub fn new(
        scans: Arc<dyn ScanRepository>,
        endpoints: Arc<dyn EndpointRepository>,
        findings: Arc<dyn FindingRepository>,
    ) -> Self {
        Self {
            scans,
            endpoints,
            findings,
        }
    }

    pub async fn execute(&self, scan_id: &str) -> Result<Value, AppError> {
        let scan = self
            .scans
            .find_by_id(scan_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("no scan with id {scan_id}")))?;

        let endpoints: Vec<Endpoint> = self
            .endpoints
            .list_for_scan(scan_id)
            .await?
            .iter()
            .map(as_endpoint)
            .collect();
        let schemes = auth_schemes(&self.findings.list_for_scan(scan_id).await?);

        Ok(tenet_spec::build(&SpecInput {
            title: &scan.target,
            version: SPEC_VERSION,
            server: target_origin(&scan.target).as_deref(),
            endpoints: &endpoints,
            auth_schemes: &schemes,
        }))
    }
}

fn target_origin(target: &str) -> Option<String> {
    let parsed = url::Url::parse(target).ok()?;
    let origin = parsed.origin();
    if origin.is_tuple() {
        return Some(origin.ascii_serialization());
    }
    None
}

fn as_endpoint(record: &EndpointRecord) -> Endpoint {
    Endpoint::new(
        HttpMethod::from_name(&record.method),
        record.path.clone(),
        record.source.clone(),
        record.confidence,
    )
    .with_base_url(record.base_url.clone())
}

fn auth_schemes(findings: &[FindingRecord]) -> Vec<AuthScheme> {
    let mut schemes: Vec<AuthScheme> = Vec::new();
    for finding in findings {
        if finding.kind != "auth" {
            continue;
        }
        let Some(scheme) = AuthScheme::from_name(&finding.name) else {
            continue;
        };
        if !schemes.contains(&scheme) {
            schemes.push(scheme);
        }
    }
    schemes
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use tenet_types::AuthScheme;

    use crate::domain::artifact::FindingRecord;

    use super::{auth_schemes, target_origin};

    #[test]
    fn the_scan_target_becomes_the_spec_server() {
        let origin = target_origin("https://shopee.co.id/some/page?x=1");
        assert_eq!(origin, Some("https://shopee.co.id".to_owned()));
    }

    #[test]
    fn a_target_that_is_not_a_url_has_no_origin() {
        assert_eq!(target_origin("com.example.app"), None);
    }

    fn finding(kind: &str, name: &str) -> FindingRecord {
        FindingRecord {
            kind: kind.to_owned(),
            name: name.to_owned(),
            value: None,
            severity: "info".to_owned(),
            confidence: 0.8,
            evidence: None,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn only_auth_findings_become_schemes() {
        let findings = vec![finding("technology", "bearer"), finding("auth", "bearer")];
        assert_eq!(auth_schemes(&findings), vec![AuthScheme::Bearer]);
    }

    #[test]
    fn a_repeated_scheme_is_listed_once() {
        let findings = vec![finding("auth", "cookie"), finding("auth", "cookie")];
        assert_eq!(auth_schemes(&findings).len(), 1);
    }

    #[test]
    fn an_unknown_scheme_name_is_ignored() {
        assert!(auth_schemes(&[finding("auth", "auth_endpoint")]).is_empty());
    }
}
