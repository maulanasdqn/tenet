use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ScanStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
}

impl ScanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ScanStatus::Queued => "queued",
            ScanStatus::Running => "running",
            ScanStatus::Succeeded => "succeeded",
            ScanStatus::Failed => "failed",
        }
    }

    pub fn from_name(raw: &str) -> Self {
        match raw {
            "running" => ScanStatus::Running,
            "succeeded" => ScanStatus::Succeeded,
            "failed" => ScanStatus::Failed,
            _ => ScanStatus::Queued,
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, ScanStatus::Succeeded | ScanStatus::Failed)
    }
}
