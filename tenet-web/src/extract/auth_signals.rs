use tenet_types::{AuthScheme, Endpoint, Finding};

use crate::page::{PageSnapshot, ScriptAsset};

const SESSION_COOKIE_MARKERS: &[&str] = &["session", "sid", "auth", "token", "jwt"];
const LOGIN_PATH_MARKERS: &[&str] = &[
    "/login",
    "/signin",
    "/sign-in",
    "/auth",
    "/oauth",
    "/token",
    "/session",
    "/register",
    "/signup",
];

pub fn from_challenge(page: &PageSnapshot, findings: &mut Vec<Finding>) {
    let Some(challenge) = page.header("www-authenticate") else {
        return;
    };
    let lower = challenge.to_lowercase();
    if lower.starts_with("bearer") {
        findings.push(scheme(AuthScheme::Bearer, 0.95, "www-authenticate"));
    }
    if lower.starts_with("basic") {
        findings.push(scheme(AuthScheme::Basic, 0.95, "www-authenticate"));
    }
}

pub fn from_cookies(page: &PageSnapshot, findings: &mut Vec<Finding>) {
    for cookie in page.cookies() {
        let name = cookie.split('=').next().unwrap_or(cookie).to_lowercase();
        if SESSION_COOKIE_MARKERS
            .iter()
            .any(|marker| name.contains(marker))
        {
            findings.push(Finding::auth(
                AuthScheme::Cookie.as_str(),
                Some(name),
                0.85,
                "set-cookie",
            ));
        }
    }
}

pub fn from_source(page: &PageSnapshot, scripts: &[ScriptAsset], findings: &mut Vec<Finding>) {
    let corpus = corpus(page, scripts);
    if corpus.contains("authorization") && corpus.contains("bearer") {
        findings.push(scheme(
            AuthScheme::Bearer,
            0.8,
            "authorization header in source",
        ));
    }
    if corpus.contains("x-api-key") || corpus.contains("apikey") {
        findings.push(scheme(AuthScheme::ApiKey, 0.7, "api key header in source"));
    }
    if corpus.contains("response_type=code")
        || corpus.contains("/oauth/authorize")
        || corpus.contains("grant_type")
    {
        findings.push(scheme(
            AuthScheme::OAuth2,
            0.75,
            "oauth parameters in source",
        ));
    }
    if corpus.contains("localstorage") && corpus.contains("token") {
        findings.push(Finding::auth(
            "token_storage",
            Some("localStorage".to_owned()),
            0.6,
            "token read from local storage",
        ));
    }
}

pub fn from_endpoints(endpoints: &[Endpoint], findings: &mut Vec<Finding>) {
    for endpoint in endpoints {
        let path = endpoint.path.to_lowercase();
        if LOGIN_PATH_MARKERS
            .iter()
            .any(|marker| path.contains(marker))
        {
            findings.push(Finding::auth(
                "auth_endpoint",
                Some(endpoint.identity()),
                endpoint.confidence,
                endpoint.source.clone(),
            ));
        }
    }
}

fn corpus(page: &PageSnapshot, scripts: &[ScriptAsset]) -> String {
    let mut text = page.body.to_lowercase();
    for script in scripts {
        text.push('\n');
        text.push_str(&script.body.to_lowercase());
    }
    text
}

fn scheme(value: AuthScheme, confidence: f32, evidence: &str) -> Finding {
    Finding::auth(value.as_str(), None, confidence, evidence)
}

#[cfg(test)]
mod tests {
    use tenet_types::{Endpoint, Finding, HttpMethod};

    use crate::page::PageSnapshot;

    use super::{from_cookies, from_endpoints};

    #[test]
    fn an_unrelated_cookie_is_not_a_session() {
        let page = PageSnapshot::new(
            "https://example.com/",
            200,
            vec![("Set-Cookie".to_owned(), "locale=en".to_owned())],
            "",
        );
        let mut findings: Vec<Finding> = Vec::new();
        from_cookies(&page, &mut findings);
        assert!(findings.is_empty());
    }

    #[test]
    fn a_plain_path_is_not_an_auth_endpoint() {
        let endpoints = vec![Endpoint::new(HttpMethod::Get, "/api/items", "app.js", 0.8)];
        let mut findings: Vec<Finding> = Vec::new();
        from_endpoints(&endpoints, &mut findings);
        assert!(findings.is_empty());
    }
}
