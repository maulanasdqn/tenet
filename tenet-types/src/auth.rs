use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AuthScheme {
    Cookie,
    Bearer,
    ApiKey,
    Basic,
    OAuth2,
}

impl AuthScheme {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuthScheme::Cookie => "cookie",
            AuthScheme::Bearer => "bearer",
            AuthScheme::ApiKey => "api_key",
            AuthScheme::Basic => "basic",
            AuthScheme::OAuth2 => "oauth2",
        }
    }

    pub fn from_name(raw: &str) -> Option<Self> {
        match raw {
            "cookie" => Some(AuthScheme::Cookie),
            "bearer" => Some(AuthScheme::Bearer),
            "api_key" => Some(AuthScheme::ApiKey),
            "basic" => Some(AuthScheme::Basic),
            "oauth2" => Some(AuthScheme::OAuth2),
            _ => None,
        }
    }
}
