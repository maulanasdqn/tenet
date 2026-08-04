use serde_json::{json, Map, Value};
use tenet_types::AuthScheme;

pub fn security_schemes(schemes: &[AuthScheme]) -> Map<String, Value> {
    let mut document: Map<String, Value> = Map::new();
    for scheme in schemes {
        document.insert(scheme.as_str().to_owned(), definition(*scheme));
    }
    document
}

fn definition(scheme: AuthScheme) -> Value {
    match scheme {
        AuthScheme::Bearer => json!({ "type": "http", "scheme": "bearer" }),
        AuthScheme::Basic => json!({ "type": "http", "scheme": "basic" }),
        AuthScheme::ApiKey => {
            json!({ "type": "apiKey", "in": "header", "name": "X-API-Key" })
        }
        AuthScheme::Cookie => {
            json!({ "type": "apiKey", "in": "cookie", "name": "session" })
        }
        AuthScheme::OAuth2 => json!({ "type": "oauth2", "flows": {} }),
    }
}

#[cfg(test)]
mod tests {
    use tenet_types::AuthScheme;

    use super::security_schemes;

    #[test]
    fn an_api_key_is_declared_as_a_header() {
        let document = security_schemes(&[AuthScheme::ApiKey]);
        assert_eq!(document["api_key"]["in"], "header");
    }

    #[test]
    fn a_cookie_scheme_is_declared_in_the_cookie_slot() {
        let document = security_schemes(&[AuthScheme::Cookie]);
        assert_eq!(document["cookie"]["in"], "cookie");
    }

    #[test]
    fn no_schemes_produce_an_empty_map() {
        assert!(security_schemes(&[]).is_empty());
    }
}
