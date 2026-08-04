use std::collections::HashMap;

use tenet_types::{Endpoint, HttpMethod};

use crate::extract::normalize::split_target;
use crate::extract::patterns::{ABSOLUTE_LITERAL, FETCH_CALL, METHOD_CALL, PATH_LITERAL};

const METHOD_CALL_CONFIDENCE: f32 = 0.85;
const FETCH_CONFIDENCE: f32 = 0.7;
const LITERAL_CONFIDENCE: f32 = 0.45;

pub fn endpoints(source: &str, source_name: &str) -> Vec<Endpoint> {
    let mut found = Vec::new();
    collect_method_calls(source, source_name, &mut found);
    collect_fetch_calls(source, source_name, &mut found);
    collect_literals(source, source_name, &mut found);
    dedupe(found)
}

fn collect_method_calls(source: &str, source_name: &str, out: &mut Vec<Endpoint>) {
    let Some(pattern) = METHOD_CALL.as_ref() else {
        return;
    };
    for capture in pattern.captures_iter(source) {
        let (Some(method), Some(target)) = (capture.get(1), capture.get(2)) else {
            continue;
        };
        push(
            out,
            HttpMethod::from_name(method.as_str()),
            target.as_str(),
            source_name,
            METHOD_CALL_CONFIDENCE,
        );
    }
}

fn collect_fetch_calls(source: &str, source_name: &str, out: &mut Vec<Endpoint>) {
    let Some(pattern) = FETCH_CALL.as_ref() else {
        return;
    };
    for capture in pattern.captures_iter(source) {
        let Some(target) = capture.get(1) else {
            continue;
        };
        push(
            out,
            HttpMethod::Get,
            target.as_str(),
            source_name,
            FETCH_CONFIDENCE,
        );
    }
}

fn collect_literals(source: &str, source_name: &str, out: &mut Vec<Endpoint>) {
    for pattern in [PATH_LITERAL.as_ref(), ABSOLUTE_LITERAL.as_ref()]
        .into_iter()
        .flatten()
    {
        for capture in pattern.captures_iter(source) {
            let Some(target) = capture.get(1) else {
                continue;
            };
            push(
                out,
                HttpMethod::Get,
                target.as_str(),
                source_name,
                LITERAL_CONFIDENCE,
            );
        }
    }
}

fn push(
    out: &mut Vec<Endpoint>,
    method: HttpMethod,
    raw: &str,
    source_name: &str,
    confidence: f32,
) {
    let Some((base_url, path)) = split_target(raw) else {
        return;
    };
    out.push(Endpoint::new(method, path, source_name, confidence).with_base_url(base_url));
}

fn dedupe(found: Vec<Endpoint>) -> Vec<Endpoint> {
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
    result.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.method.as_str().cmp(right.method.as_str()))
    });
    result
}

#[cfg(test)]
mod tests {
    use tenet_types::HttpMethod;

    use super::endpoints;

    const BUNDLE: &str = r#"
        const base = "https://api.example.com/v1/accounts";
        axios.post("/api/v1/sessions", body);
        client.delete(`/api/v1/sessions/${sessionId}`);
        fetch("/api/v1/profile").then(read);
        const asset = "/static/app.js";
        const graph = "/graphql";
    "#;

    fn paths() -> Vec<String> {
        endpoints(BUNDLE, "bundle.js")
            .into_iter()
            .map(|endpoint| endpoint.identity())
            .collect()
    }

    #[test]
    fn a_method_call_keeps_its_verb() {
        assert!(paths().contains(&"post /api/v1/sessions".to_owned()));
    }

    #[test]
    fn a_fetch_call_defaults_to_get() {
        assert!(paths().contains(&"get /api/v1/profile".to_owned()));
    }

    #[test]
    fn a_concatenated_argument_is_not_mistaken_for_a_whole_path() {
        let found = endpoints(
            r#"fetch("/ap" + rest); axios.get("/gate" + tail);"#,
            "inline",
        );
        assert!(found.is_empty());
    }

    #[test]
    fn a_call_with_options_after_the_path_is_still_read() {
        let found = endpoints(r#"fetch("/api/keep", { method: "GET" })"#, "inline");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].path, "/api/keep");
    }

    #[test]
    fn a_template_literal_is_parameterised() {
        assert!(paths().contains(&"delete /api/v1/sessions/{sessionId}".to_owned()));
    }

    #[test]
    fn an_absolute_literal_keeps_its_origin() {
        let found = endpoints(BUNDLE, "bundle.js");
        let account = found
            .iter()
            .find(|endpoint| endpoint.path == "/v1/accounts")
            .map(|endpoint| endpoint.base_url.clone());
        assert_eq!(account, Some(Some("https://api.example.com".to_owned())));
    }

    #[test]
    fn a_static_asset_is_not_an_endpoint() {
        assert!(!paths().iter().any(|identity| identity.contains("app.js")));
    }

    #[test]
    fn a_bare_graphql_path_is_kept() {
        assert!(paths().contains(&"get /graphql".to_owned()));
    }

    #[test]
    fn a_repeated_endpoint_keeps_the_strongest_signal() {
        let found = endpoints(r#"fetch("/api/x"); axios.get("/api/x");"#, "inline");
        let matched: Vec<&tenet_types::Endpoint> = found
            .iter()
            .filter(|endpoint| endpoint.path == "/api/x")
            .collect();
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].method, HttpMethod::Get);
        assert!(matched[0].confidence > 0.8);
    }
}
