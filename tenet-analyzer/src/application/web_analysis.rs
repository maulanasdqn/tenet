use std::collections::HashMap;

use tenet_types::{Endpoint, Finding};
use tenet_web::{PageSnapshot, ScriptAsset};

use crate::domain::work::{Analysis, ArtifactRecord};

pub struct AnalysisInput {
    pub page: PageSnapshot,
    pub scripts: Vec<ScriptAsset>,
    pub artifacts: Vec<ArtifactRecord>,
    pub observed_endpoints: Vec<Endpoint>,
    pub observed_findings: Vec<Finding>,
}

pub fn analyse(input: AnalysisInput) -> Analysis {
    let mut endpoints = static_endpoints(&input.page, &input.scripts);
    endpoints.extend(input.observed_endpoints);
    let endpoints = dedupe_endpoints(endpoints);

    let mut findings = tenet_web::detect(&input.page, &input.scripts);
    findings.extend(tenet_web::auth_findings(
        &input.page,
        &input.scripts,
        &endpoints,
    ));
    findings.extend(input.observed_findings);

    Analysis {
        artifacts: input.artifacts,
        findings: dedupe_findings(findings),
        endpoints,
    }
}

fn static_endpoints(page: &PageSnapshot, scripts: &[ScriptAsset]) -> Vec<Endpoint> {
    let mut found = tenet_web::endpoints(&page.body, &page.url);
    for script in scripts {
        found.extend(tenet_web::endpoints(&script.body, &script.url));
    }
    found
}

fn dedupe_endpoints(found: Vec<Endpoint>) -> Vec<Endpoint> {
    let mut best: HashMap<String, Endpoint> = HashMap::new();
    for endpoint in found {
        let identity = endpoint.identity();
        let keep = best
            .get(&identity)
            .is_none_or(|existing| endpoint.confidence > existing.confidence);
        if keep {
            best.insert(identity, endpoint);
        }
    }
    let mut result: Vec<Endpoint> = best.into_values().collect();
    result.sort_by_key(Endpoint::identity);
    result
}

fn dedupe_findings(found: Vec<Finding>) -> Vec<Finding> {
    let mut best: HashMap<String, Finding> = HashMap::new();
    for finding in found {
        let identity = finding.identity();
        let keep = best
            .get(&identity)
            .is_none_or(|existing| finding.confidence > existing.confidence);
        if keep {
            best.insert(identity, finding);
        }
    }
    let mut result: Vec<Finding> = best.into_values().collect();
    result.sort_by_key(Finding::identity);
    result
}

#[cfg(test)]
mod tests {
    use tenet_types::{Endpoint, HttpMethod};
    use tenet_web::PageSnapshot;

    use super::{analyse, AnalysisInput};

    fn input(observed: Vec<Endpoint>) -> AnalysisInput {
        AnalysisInput {
            page: PageSnapshot::new(
                "https://example.com/",
                200,
                Vec::new(),
                r#"<script>fetch("/api/x")</script>"#,
            ),
            scripts: Vec::new(),
            artifacts: Vec::new(),
            observed_endpoints: observed,
            observed_findings: Vec::new(),
        }
    }

    #[test]
    fn a_static_endpoint_is_found_without_any_observation() {
        let analysis = analyse(input(Vec::new()));
        assert!(analysis
            .endpoints
            .iter()
            .any(|endpoint| endpoint.path == "/api/x"));
    }

    #[test]
    fn an_observed_call_outranks_the_same_path_found_statically() {
        let observed = vec![Endpoint::new(HttpMethod::Get, "/api/x", "runtime", 0.95)];
        let analysis = analyse(input(observed));
        let found: Vec<&Endpoint> = analysis
            .endpoints
            .iter()
            .filter(|endpoint| endpoint.path == "/api/x")
            .collect();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].source, "runtime");
    }

    #[test]
    fn an_observed_call_that_static_analysis_missed_is_kept() {
        let observed = vec![Endpoint::new(
            HttpMethod::Post,
            "/hidden/route",
            "runtime",
            0.95,
        )];
        let analysis = analyse(input(observed));
        assert!(analysis
            .endpoints
            .iter()
            .any(|endpoint| endpoint.path == "/hidden/route"));
    }
}
