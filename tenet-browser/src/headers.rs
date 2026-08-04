use chromiumoxide::cdp::browser_protocol::network::{Headers, ResourceType};

const API_KEY_HEADERS: [&str; 3] = ["x-api-key", "apikey", "api-key"];

pub fn as_pairs(headers: &Headers) -> Vec<(String, String)> {
    let Some(map) = headers.inner().as_object() else {
        return Vec::new();
    };
    map.iter()
        .map(|(key, value)| {
            let text = value
                .as_str()
                .map_or_else(|| value.to_string(), ToOwned::to_owned);
            (key.to_lowercase(), text)
        })
        .collect()
}

pub fn header_value(headers: &[(String, String)], name: &str) -> Option<String> {
    headers
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.clone())
}

pub fn api_key_name(headers: &[(String, String)]) -> Option<String> {
    headers
        .iter()
        .map(|(key, _)| key)
        .find(|key| API_KEY_HEADERS.contains(&key.as_str()))
        .cloned()
}

pub fn label(kind: Option<&ResourceType>) -> String {
    match kind {
        Some(ResourceType::Xhr) => "xhr",
        Some(ResourceType::Fetch) => "fetch",
        Some(ResourceType::Document) => "document",
        Some(ResourceType::Script) => "script",
        Some(_) => "other",
        None => "unknown",
    }
    .to_owned()
}

#[cfg(test)]
mod tests {
    use chromiumoxide::cdp::browser_protocol::network::{Headers, ResourceType};
    use serde_json::json;

    use super::{api_key_name, as_pairs, header_value, label};

    #[test]
    fn header_names_are_lowercased() {
        let headers = Headers::new(json!({ "Authorization": "Bearer x", "X-Api-Key": "k" }));
        let pairs = as_pairs(&headers);
        assert_eq!(
            header_value(&pairs, "authorization"),
            Some("Bearer x".to_owned())
        );
        assert_eq!(api_key_name(&pairs), Some("x-api-key".to_owned()));
    }

    #[test]
    fn headers_that_are_not_an_object_yield_nothing() {
        assert!(as_pairs(&Headers::new(json!("nope"))).is_empty());
    }

    #[test]
    fn a_missing_header_has_no_value() {
        let pairs = as_pairs(&Headers::new(json!({ "accept": "*/*" })));
        assert_eq!(header_value(&pairs, "authorization"), None);
        assert_eq!(api_key_name(&pairs), None);
    }

    #[test]
    fn every_resource_type_gets_a_stable_label() {
        assert_eq!(label(Some(&ResourceType::Xhr)), "xhr");
        assert_eq!(label(Some(&ResourceType::Image)), "other");
        assert_eq!(label(None), "unknown");
    }
}
