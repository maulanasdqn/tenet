use std::collections::HashMap;

use tenet_types::{Endpoint, Finding};

use crate::extract::auth_signals::{from_challenge, from_cookies, from_endpoints, from_source};
use crate::page::{PageSnapshot, ScriptAsset};

pub fn auth_findings(
    page: &PageSnapshot,
    scripts: &[ScriptAsset],
    endpoints: &[Endpoint],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    from_challenge(page, &mut findings);
    from_cookies(page, &mut findings);
    from_source(page, scripts, &mut findings);
    from_endpoints(endpoints, &mut findings);
    dedupe(findings)
}

fn dedupe(findings: Vec<Finding>) -> Vec<Finding> {
    let mut best: HashMap<String, Finding> = HashMap::new();
    for finding in findings {
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
    use tenet_types::{Endpoint, Finding, HttpMethod};

    use crate::page::{PageSnapshot, ScriptAsset};

    use super::auth_findings;

    fn page(headers: Vec<(&str, &str)>, body: &str) -> PageSnapshot {
        PageSnapshot::new(
            "https://example.com/",
            200,
            headers
                .into_iter()
                .map(|(key, value)| (key.to_owned(), value.to_owned()))
                .collect(),
            body,
        )
    }

    fn names(findings: &[Finding]) -> Vec<String> {
        findings.iter().map(|found| found.name.clone()).collect()
    }

    #[test]
    fn a_challenge_header_names_the_scheme() {
        let found = auth_findings(
            &page(vec![("WWW-Authenticate", "Bearer realm=x")], ""),
            &[],
            &[],
        );
        assert!(names(&found).contains(&"bearer".to_owned()));
    }

    #[test]
    fn a_session_cookie_is_reported() {
        let found = auth_findings(&page(vec![("Set-Cookie", "sessionid=abc")], ""), &[], &[]);
        assert!(names(&found).contains(&"cookie".to_owned()));
    }

    #[test]
    fn a_bearer_header_in_a_bundle_is_reported() {
        let scripts = vec![ScriptAsset::new(
            "https://example.com/app.js",
            r"headers: { Authorization: `Bearer ${token}` }",
        )];
        let found = auth_findings(&page(vec![], ""), &scripts, &[]);
        assert!(names(&found).contains(&"bearer".to_owned()));
    }

    #[test]
    fn a_login_path_is_flagged_as_an_auth_endpoint() {
        let endpoints = vec![Endpoint::new(
            HttpMethod::Post,
            "/api/v1/login",
            "app.js",
            0.9,
        )];
        let found = auth_findings(&page(vec![], ""), &[], &endpoints);
        assert!(names(&found).contains(&"auth_endpoint".to_owned()));
    }

    #[test]
    fn a_repeated_scheme_is_reported_once() {
        let scripts = vec![ScriptAsset::new("https://example.com/app.js", "Bearer")];
        let found = auth_findings(
            &page(
                vec![("WWW-Authenticate", "Bearer realm=x")],
                "authorization",
            ),
            &scripts,
            &[],
        );
        let hits = names(&found)
            .iter()
            .filter(|name| *name == "bearer")
            .count();
        assert_eq!(hits, 1);
    }

    #[test]
    fn an_anonymous_page_reports_nothing() {
        assert!(auth_findings(&page(vec![], "<html></html>"), &[], &[]).is_empty());
    }
}
