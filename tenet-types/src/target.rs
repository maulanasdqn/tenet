use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum TargetKind {
    Web,
    Mobile,
}

impl TargetKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            TargetKind::Web => "web",
            TargetKind::Mobile => "mobile",
        }
    }

    pub fn from_name(raw: &str) -> Self {
        match raw {
            "mobile" => TargetKind::Mobile,
            _ => TargetKind::Web,
        }
    }
}
