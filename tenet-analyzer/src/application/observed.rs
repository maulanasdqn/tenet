use tenet_types::{AuthScheme, Endpoint, Finding, HttpMethod};

use crate::domain::work::{ObservedRequest, RenderedDocument};

const OBSERVED_CONFIDENCE: f32 = 0.95;
const OBSERVED_SOURCE: &str = "runtime";
const TOKEN_KEY_MARKERS: &[&str] = &["token", "jwt", "auth", "session"];

pub fn observed_endpoints(requests: &[ObservedRequest]) -> Vec<Endpoint> {
    requests.iter().filter_map(as_endpoint).collect()
}

pub fn observed_findings(rendered: &RenderedDocument) -> Vec<Finding> {
    let mut findings = Vec::new();
    for request in &rendered.observed {
        push_scheme(request, &mut findings);
        push_api_key(request, &mut findings);
    }
    for key in &rendered.storage_keys {
        push_storage_key(key, &mut findings);
    }
    findings
}

fn as_endpoint(request: &ObservedRequest) -> Option<Endpoint> {
    let (base_url, path) = tenet_web::split_target(&request.url)?;
    Some(
        Endpoint::new(
            HttpMethod::from_name(&request.method),
            path,
            OBSERVED_SOURCE,
            OBSERVED_CONFIDENCE,
        )
        .with_base_url(base_url),
    )
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

#[cfg(test)]
mod tests {
    use tenet_types::HttpMethod;

    use crate::domain::work::{FetchedDocument, ObservedRequest, RenderedDocument};

    use super::{observed_endpoints, observed_findings};

    fn request(method: &str, url: &str, auth: Option<&str>) -> ObservedRequest {
        ObservedRequest {
            method: method.to_owned(),
            url: url.to_owned(),
            auth_scheme: auth.map(ToOwned::to_owned),
            api_key_header: None,
        }
    }

    fn rendered(observed: Vec<ObservedRequest>, storage_keys: Vec<String>) -> RenderedDocument {
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
            storage_keys,
        }
    }

    #[test]
    fn an_observed_call_becomes_a_high_confidence_endpoint() {
        let found =
            observed_endpoints(&[request("POST", "https://api.example.com/v2/orders", None)]);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].method, HttpMethod::Post);
        assert_eq!(found[0].path, "/v2/orders");
        assert_eq!(
            found[0].base_url,
            Some("https://api.example.com".to_owned())
        );
        assert!(found[0].confidence > 0.9);
    }

    #[test]
    fn a_call_to_the_site_root_is_not_an_endpoint() {
        assert!(observed_endpoints(&[request("GET", "https://example.com/", None)]).is_empty());
    }

    #[test]
    fn an_observed_bearer_header_becomes_an_auth_finding() {
        let document = rendered(
            vec![request("GET", "https://example.com/api/me", Some("bearer"))],
            Vec::new(),
        );
        let found = observed_findings(&document);
        assert!(found.iter().any(|finding| finding.name == "bearer"));
    }

    #[test]
    fn a_token_shaped_storage_key_is_reported_with_its_real_name() {
        let document = rendered(Vec::new(), vec!["sb-access-token".to_owned()]);
        let found = observed_findings(&document);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].value, Some("sb-access-token".to_owned()));
    }

    #[test]
    fn an_unrelated_storage_key_is_ignored() {
        let document = rendered(Vec::new(), vec!["theme".to_owned()]);
        assert!(observed_findings(&document).is_empty());
    }
}
