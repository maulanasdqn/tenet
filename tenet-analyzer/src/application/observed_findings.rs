use tenet_types::{AuthScheme, Finding, FindingKind, Severity};

use crate::domain::work::{ObservedRequest, RenderedDocument};

const OBSERVED_CONFIDENCE: f32 = 0.95;
const TOKEN_KEY_MARKERS: &[&str] = &["token", "jwt", "auth", "session"];

pub fn observed_findings(rendered: &RenderedDocument) -> Vec<Finding> {
    let mut findings = Vec::new();
    for request in &rendered.observed {
        push_scheme(request, &mut findings);
        push_api_key(request, &mut findings);
        push_gated(request, &mut findings);
    }
    for key in &rendered.storage_keys {
        push_storage_key(key, &mut findings);
    }
    for module in &rendered.wasm_modules {
        push_wasm(module, &mut findings);
    }
    findings
}

fn push_scheme(request: &ObservedRequest, findings: &mut Vec<Finding>) {
    let Some(scheme) = request
        .auth_scheme
        .as_deref()
        .and_then(AuthScheme::from_name)
    else {
        return;
    };
    findings.push(Finding::auth(
        scheme.as_str(),
        None,
        OBSERVED_CONFIDENCE,
        "observed on a live request",
    ));
}

fn push_api_key(request: &ObservedRequest, findings: &mut Vec<Finding>) {
    let Some(header) = &request.api_key_header else {
        return;
    };
    findings.push(Finding::auth(
        AuthScheme::ApiKey.as_str(),
        Some(header.clone()),
        OBSERVED_CONFIDENCE,
        "observed on a live request",
    ));
}

fn push_storage_key(key: &str, findings: &mut Vec<Finding>) {
    let lower = key.to_lowercase();
    if !TOKEN_KEY_MARKERS
        .iter()
        .any(|marker| lower.contains(marker))
    {
        return;
    }
    findings.push(Finding::auth(
        "token_storage",
        Some(key.to_owned()),
        OBSERVED_CONFIDENCE,
        "localStorage key read from the rendered page",
    ));
}

fn push_gated(request: &ObservedRequest, findings: &mut Vec<Finding>) {
    let Some(status) = request.status.filter(|_| request.gated) else {
        return;
    };
    let Some((_, path)) = tenet_web::split_target(&request.url) else {
        return;
    };
    findings.push(Finding {
        kind: FindingKind::Auth,
        name: "gated_endpoint".to_owned(),
        value: Some(format!("{} {path}", request.method.to_uppercase())),
        severity: Severity::Medium,
        confidence: OBSERVED_CONFIDENCE,
        evidence: Some(format!(
            "the gateway answered {status} without a valid token"
        )),
    });
}

fn push_wasm(module: &str, findings: &mut Vec<Finding>) {
    findings.push(Finding {
        kind: FindingKind::Technology,
        name: "WebAssembly module".to_owned(),
        value: Some("wasm".to_owned()),
        severity: Severity::Info,
        confidence: 0.9,
        evidence: Some(format!("loaded {module}")),
    });
}

#[cfg(test)]
mod tests {
    use crate::domain::work::{FetchedDocument, ObservedRequest, RenderedDocument};

    use super::observed_findings;

    fn request(status: u16, gated: bool, url: &str, auth: Option<&str>) -> ObservedRequest {
        ObservedRequest {
            method: "get".to_owned(),
            url: url.to_owned(),
            status: Some(status),
            gated,
            auth_scheme: auth.map(ToOwned::to_owned),
            api_key_header: None,
        }
    }

    fn rendered(observed: Vec<ObservedRequest>) -> RenderedDocument {
        RenderedDocument {
            document: FetchedDocument {
                url: "https://example.com/".to_owned(),
                status: 200,
                headers: Vec::new(),
                body: String::new(),
                sha256: String::new(),
                byte_size: 0,
            },
            observed,
            storage_keys: Vec::new(),
            wasm_modules: Vec::new(),
        }
    }

    #[test]
    fn an_observed_bearer_header_becomes_an_auth_finding() {
        let document = rendered(vec![request(
            200,
            false,
            "https://x.test/api/me",
            Some("bearer"),
        )]);
        assert!(observed_findings(&document)
            .iter()
            .any(|finding| finding.name == "bearer"));
    }

    #[test]
    fn a_forbidden_api_call_becomes_a_gated_endpoint_finding() {
        let document = rendered(vec![request(
            403,
            true,
            "https://shopee.co.id/api/v4/recommend/recommend",
            None,
        )]);
        let found = observed_findings(&document);
        let gate = found
            .iter()
            .find(|finding| finding.name == "gated_endpoint");
        assert_eq!(
            gate.and_then(|finding| finding.value.clone()),
            Some("GET /api/v4/recommend/recommend".to_owned())
        );
    }

    #[test]
    fn a_healthy_api_call_is_not_gated() {
        let document = rendered(vec![request(200, false, "https://x.test/api/ok", None)]);
        assert!(!observed_findings(&document)
            .iter()
            .any(|finding| finding.name == "gated_endpoint"));
    }

    #[test]
    fn a_token_shaped_storage_key_is_reported_by_name() {
        let mut document = rendered(Vec::new());
        document.storage_keys = vec!["sb-access-token".to_owned(), "theme".to_owned()];
        let found = observed_findings(&document);
        let keys: Vec<&Option<String>> = found
            .iter()
            .filter(|finding| finding.name == "token_storage")
            .map(|finding| &finding.value)
            .collect();
        assert_eq!(keys, vec![&Some("sb-access-token".to_owned())]);
    }

    #[test]
    fn a_loaded_wasm_module_is_reported() {
        let mut document = rendered(Vec::new());
        document.wasm_modules = vec!["https://x.test/fp/sensor.wasm".to_owned()];
        assert!(observed_findings(&document)
            .iter()
            .any(|finding| finding.name == "WebAssembly module"));
    }
}
