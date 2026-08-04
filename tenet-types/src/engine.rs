use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Engine {
    Http,
    Browser,
}

impl Engine {
    pub fn as_str(&self) -> &'static str {
        match self {
            Engine::Http => "http",
            Engine::Browser => "browser",
        }
    }

    pub fn from_name(raw: &str) -> Self {
        match raw {
            "browser" => Engine::Browser,
            _ => Engine::Http,
        }
    }

    pub fn renders(&self) -> bool {
        matches!(self, Engine::Browser)
    }
}

#[cfg(test)]
mod tests {
    use super::Engine;

    #[test]
    fn an_unknown_name_falls_back_to_http() {
        assert_eq!(Engine::from_name("nonsense"), Engine::Http);
        assert_eq!(Engine::from_name("browser"), Engine::Browser);
    }

    #[test]
    fn only_the_browser_engine_renders() {
        assert!(Engine::Browser.renders());
        assert!(!Engine::Http.renders());
    }
}
