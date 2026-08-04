use serde_json::{json, Value};
use tenet_types::Endpoint;

pub fn operation(endpoint: &Endpoint, server: Option<&str>) -> Value {
    let mut document = json!({
        "operationId": operation_id(endpoint),
        "summary": format!("{} {}", endpoint.method.as_str().to_uppercase(), endpoint.path),
        "responses": { "default": { "description": "observed during reverse engineering" } },
        "x-tenet-confidence": endpoint.confidence,
        "x-tenet-source": endpoint.source,
    });

    let parameters = path_parameters(&endpoint.path);
    if !parameters.is_empty() {
        document["parameters"] = Value::Array(parameters);
    }
    if let Some(base_url) = foreign_base_url(endpoint, server) {
        document["servers"] = json!([{ "url": base_url }]);
    }

    document
}

fn foreign_base_url(endpoint: &Endpoint, server: Option<&str>) -> Option<String> {
    let base_url = endpoint.base_url.as_ref()?;
    if server.is_some_and(|server| server == base_url) {
        return None;
    }
    Some(base_url.clone())
}

fn operation_id(endpoint: &Endpoint) -> String {
    let slug: String = endpoint
        .path
        .chars()
        .map(|letter| {
            if letter.is_alphanumeric() {
                letter
            } else {
                '_'
            }
        })
        .collect();
    format!("{}{}", endpoint.method.as_str(), slug)
}

fn path_parameters(path: &str) -> Vec<Value> {
    path.split('/')
        .filter_map(|segment| segment.strip_prefix('{'))
        .filter_map(|segment| segment.strip_suffix('}'))
        .map(|name| {
            json!({
                "name": name,
                "in": "path",
                "required": true,
                "schema": { "type": "string" },
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use tenet_types::{Endpoint, HttpMethod};

    use super::operation;

    #[test]
    fn a_templated_segment_becomes_a_path_parameter() {
        let endpoint = Endpoint::new(HttpMethod::Get, "/api/users/{id}", "app.js", 0.8);
        let document = operation(&endpoint, None);
        assert_eq!(document["parameters"][0]["name"], "id");
        assert_eq!(document["parameters"][0]["in"], "path");
    }

    #[test]
    fn a_plain_path_has_no_parameters() {
        let endpoint = Endpoint::new(HttpMethod::Get, "/api/users", "app.js", 0.8);
        assert!(operation(&endpoint, None).get("parameters").is_none());
    }

    #[test]
    fn the_operation_id_is_stable_and_safe() {
        let endpoint = Endpoint::new(HttpMethod::Post, "/api/v1/users", "app.js", 0.8);
        assert_eq!(
            operation(&endpoint, None)["operationId"],
            "post_api_v1_users"
        );
    }

    #[test]
    fn an_endpoint_on_another_origin_carries_its_own_server() {
        let endpoint = Endpoint::new(HttpMethod::Get, "/pixel", "runtime", 0.95)
            .with_base_url(Some("https://tracker.example".to_owned()));
        let document = operation(&endpoint, Some("https://shop.example"));
        assert_eq!(document["servers"][0]["url"], "https://tracker.example");
    }

    #[test]
    fn an_endpoint_on_the_spec_server_does_not_repeat_it() {
        let endpoint = Endpoint::new(HttpMethod::Get, "/api/x", "runtime", 0.95)
            .with_base_url(Some("https://shop.example".to_owned()));
        let document = operation(&endpoint, Some("https://shop.example"));
        assert!(document.get("servers").is_none());
    }

    #[test]
    fn the_confidence_is_carried_into_the_document() {
        let endpoint = Endpoint::new(HttpMethod::Get, "/api/users", "app.js", 0.42);
        let confidence = operation(&endpoint, None)["x-tenet-confidence"].as_f64();
        assert!(confidence.is_some_and(|value| value > 0.41 && value < 0.43));
    }
}
