use std::collections::HashMap;

use tenet_types::Finding;

use crate::fingerprint::signatures::{Signature, Source, SIGNATURES};
use crate::page::{PageSnapshot, ScriptAsset};

pub fn detect(page: &PageSnapshot, scripts: &[ScriptAsset]) -> Vec<Finding> {
    let haystack = Haystack::build(page, scripts);
    let mut best: HashMap<&'static str, Signature> = HashMap::new();

    for signature in SIGNATURES {
        if !haystack.matches(signature) {
            continue;
        }
        let entry = best.entry(signature.name).or_insert(*signature);
        if signature.confidence > entry.confidence {
            *entry = *signature;
        }
    }

    let mut findings: Vec<Finding> = best.values().map(as_finding).collect();
    findings.sort_by(|left, right| left.name.cmp(&right.name));
    findings
}

fn as_finding(signature: &Signature) -> Finding {
    Finding::technology(
        signature.name,
        signature.category,
        signature.confidence,
        signature.source.label(),
    )
}

struct Haystack {
    headers: Vec<(String, String)>,
    cookies: String,
    body: String,
    script_urls: String,
}

impl Haystack {
    fn build(page: &PageSnapshot, scripts: &[ScriptAsset]) -> Self {
        Self {
            headers: page
                .headers
                .iter()
                .map(|(key, value)| (key.to_lowercase(), value.to_lowercase()))
                .collect(),
            cookies: page.cookies().join("\n").to_lowercase(),
            body: page.body.to_lowercase(),
            script_urls: scripts
                .iter()
                .map(|script| script.url.as_str())
                .collect::<Vec<&str>>()
                .join("\n")
                .to_lowercase(),
        }
    }

    fn matches(&self, signature: &Signature) -> bool {
        match signature.source {
            Source::Header(name) => self.header_matches(name, signature.needle),
            Source::Cookie => self.cookies.contains(signature.needle),
            Source::Body => self.body.contains(signature.needle),
            Source::ScriptUrl => self.script_urls.contains(signature.needle),
        }
    }

    fn header_matches(&self, name: &str, needle: &str) -> bool {
        self.headers
            .iter()
            .any(|(key, value)| key == name && value.contains(needle))
    }
}

#[cfg(test)]
mod tests {
    use crate::page::{PageSnapshot, ScriptAsset};

    use super::detect;

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

    #[test]
    fn a_server_header_names_the_edge() {
        let findings = detect(&page(vec![("Server", "cloudflare")], ""), &[]);
        assert!(findings.iter().any(|found| found.name == "Cloudflare"));
    }

    #[test]
    fn a_body_marker_names_the_framework() {
        let findings = detect(&page(vec![], "<script src=/_next/static/x.js>"), &[]);
        assert!(findings.iter().any(|found| found.name == "Next.js"));
    }

    #[test]
    fn a_script_host_names_the_vendor() {
        let scripts = vec![ScriptAsset::new("https://js.stripe.com/v3", "")];
        let findings = detect(&page(vec![], ""), &scripts);
        assert!(findings.iter().any(|found| found.name == "Stripe"));
    }

    #[test]
    fn a_technology_is_reported_once() {
        let snapshot = page(vec![("Server", "cloudflare"), ("cf-ray", "abc")], "");
        let findings = detect(&snapshot, &[]);
        let hits = findings
            .iter()
            .filter(|found| found.name == "Cloudflare")
            .count();
        assert_eq!(hits, 1);
    }

    #[test]
    fn an_unremarkable_page_yields_nothing() {
        assert!(detect(&page(vec![], "<html></html>"), &[]).is_empty());
    }
}
