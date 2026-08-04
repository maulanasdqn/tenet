use tenet_types::{Endpoint, HttpMethod};

use crate::domain::work::ObservedRequest;

const OBSERVED_CONFIDENCE: f32 = 0.95;
const OBSERVED_SOURCE: &str = "runtime";

pub fn observed_endpoints(requests: &[ObservedRequest]) -> Vec<Endpoint> {
    requests.iter().filter_map(as_endpoint).collect()
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

#[cfg(test)]
mod tests {
    use tenet_types::HttpMethod;

    use crate::domain::work::ObservedRequest;

    use super::observed_endpoints;

    fn request(method: &str, url: &str) -> ObservedRequest {
        ObservedRequest {
            method: method.to_owned(),
            url: url.to_owned(),
            status: Some(200),
            gated: false,
            auth_scheme: None,
            api_key_header: None,
        }
    }

    #[test]
    fn an_observed_call_becomes_a_high_confidence_endpoint() {
        let found = observed_endpoints(&[request("POST", "https://api.example.com/v2/orders")]);
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
        assert!(observed_endpoints(&[request("GET", "https://example.com/")]).is_empty());
    }
}
