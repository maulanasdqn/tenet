mod operation;
mod security;

use serde_json::{json, Map, Value};
use tenet_types::{AuthScheme, Endpoint};

use crate::operation::operation;
use crate::security::security_schemes;

pub struct SpecInput<'a> {
    pub title: &'a str,
    pub version: &'a str,
    pub server: Option<&'a str>,
    pub endpoints: &'a [Endpoint],
    pub auth_schemes: &'a [AuthScheme],
}

pub fn build(input: &SpecInput) -> Value {
    let server = server_url(input);
    let mut spec = json!({
        "openapi": "3.1.0",
        "info": { "title": input.title, "version": input.version },
        "paths": paths(input.endpoints, server.as_deref()),
    });

    if let Some(server) = server {
        spec["servers"] = json!([{ "url": server }]);
    }

    let schemes = security_schemes(input.auth_schemes);
    if !schemes.is_empty() {
        spec["components"] = json!({ "securitySchemes": schemes });
    }

    spec
}

fn server_url(input: &SpecInput) -> Option<String> {
    if let Some(server) = input.server {
        return Some(server.to_owned());
    }
    most_common_base_url(input.endpoints)
}

fn most_common_base_url(endpoints: &[Endpoint]) -> Option<String> {
    let mut tally: Vec<(String, usize)> = Vec::new();
    for base_url in endpoints
        .iter()
        .filter_map(|endpoint| endpoint.base_url.as_ref())
    {
        match tally.iter_mut().find(|(seen, _)| seen == base_url) {
            Some((_, count)) => *count += 1,
            None => tally.push((base_url.clone(), 1)),
        }
    }
    tally
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(base_url, _)| base_url)
}

fn paths(endpoints: &[Endpoint], server: Option<&str>) -> Map<String, Value> {
    let mut paths: Map<String, Value> = Map::new();
    for endpoint in endpoints {
        let entry = paths
            .entry(endpoint.path.clone())
            .or_insert_with(|| json!({}));
        if let Some(methods) = entry.as_object_mut() {
            methods.insert(
                endpoint.method.as_str().to_owned(),
                operation(endpoint, server),
            );
        }
    }
    paths
}

#[cfg(test)]
mod tests {
    use tenet_types::{AuthScheme, Endpoint, HttpMethod};

    use super::{build, SpecInput};

    fn spec(endpoints: &[Endpoint], schemes: &[AuthScheme]) -> serde_json::Value {
        build(&SpecInput {
            title: "example.com",
            version: "0.1.0",
            server: None,
            endpoints,
            auth_schemes: schemes,
        })
    }

    #[test]
    fn every_endpoint_lands_under_its_path() {
        let endpoints = vec![
            Endpoint::new(HttpMethod::Get, "/api/users", "app.js", 0.8),
            Endpoint::new(HttpMethod::Post, "/api/users", "app.js", 0.9),
        ];
        let document = spec(&endpoints, &[]);
        let methods = &document["paths"]["/api/users"];
        assert!(methods["get"].is_object());
        assert!(methods["post"].is_object());
    }

    #[test]
    fn a_base_url_becomes_the_server() {
        let endpoints = vec![Endpoint::new(HttpMethod::Get, "/v1/ping", "app.js", 0.8)
            .with_base_url(Some("https://api.example.com".to_owned()))];
        let document = spec(&endpoints, &[]);
        assert_eq!(document["servers"][0]["url"], "https://api.example.com");
    }

    #[test]
    fn the_busiest_origin_wins_over_a_stray_tracker() {
        let origin = |host: &str, path: &str| {
            Endpoint::new(HttpMethod::Get, path, "runtime", 0.95)
                .with_base_url(Some(host.to_owned()))
        };
        let endpoints = vec![
            origin("https://ads.example", "/activity"),
            origin("https://shop.example", "/api/a"),
            origin("https://shop.example", "/api/b"),
        ];
        let document = spec(&endpoints, &[]);
        assert_eq!(document["servers"][0]["url"], "https://shop.example");
    }

    #[test]
    fn a_detected_scheme_becomes_a_security_scheme() {
        let document = spec(&[], &[AuthScheme::Bearer]);
        assert_eq!(
            document["components"]["securitySchemes"]["bearer"]["scheme"],
            "bearer"
        );
    }

    #[test]
    fn an_empty_scan_still_produces_a_valid_document() {
        let document = spec(&[], &[]);
        assert_eq!(document["openapi"], "3.1.0");
        assert!(document["paths"].is_object());
        assert!(document.get("components").is_none());
    }
}
