use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::Get => "get",
            HttpMethod::Post => "post",
            HttpMethod::Put => "put",
            HttpMethod::Patch => "patch",
            HttpMethod::Delete => "delete",
            HttpMethod::Head => "head",
            HttpMethod::Options => "options",
        }
    }

    pub fn from_name(raw: &str) -> Self {
        match raw.to_ascii_lowercase().as_str() {
            "post" => HttpMethod::Post,
            "put" => HttpMethod::Put,
            "patch" => HttpMethod::Patch,
            "delete" => HttpMethod::Delete,
            "head" => HttpMethod::Head,
            "options" => HttpMethod::Options,
            _ => HttpMethod::Get,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Endpoint {
    pub method: HttpMethod,
    pub path: String,
    pub base_url: Option<String>,
    pub source: String,
    pub confidence: f32,
}

impl Endpoint {
    pub fn new(
        method: HttpMethod,
        path: impl Into<String>,
        source: impl Into<String>,
        confidence: f32,
    ) -> Self {
        Self {
            method,
            path: path.into(),
            base_url: None,
            source: source.into(),
            confidence: crate::clamp_confidence(confidence),
        }
    }

    #[must_use]
    pub fn with_base_url(mut self, base_url: Option<String>) -> Self {
        self.base_url = base_url;
        self
    }

    pub fn identity(&self) -> String {
        format!("{} {}", self.method.as_str(), self.path)
    }
}
