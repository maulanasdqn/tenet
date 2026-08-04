use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum FindingKind {
    Technology,
    Endpoint,
    Auth,
    Secret,
}

impl FindingKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            FindingKind::Technology => "technology",
            FindingKind::Endpoint => "endpoint",
            FindingKind::Auth => "auth",
            FindingKind::Secret => "secret",
        }
    }

    pub fn from_name(raw: &str) -> Self {
        match raw {
            "endpoint" => FindingKind::Endpoint,
            "auth" => FindingKind::Auth,
            "secret" => FindingKind::Secret,
            _ => FindingKind::Technology,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Info => "info",
            Severity::Low => "low",
            Severity::Medium => "medium",
            Severity::High => "high",
            Severity::Critical => "critical",
        }
    }

    pub fn from_name(raw: &str) -> Self {
        match raw {
            "low" => Severity::Low,
            "medium" => Severity::Medium,
            "high" => Severity::High,
            "critical" => Severity::Critical,
            _ => Severity::Info,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Finding {
    pub kind: FindingKind,
    pub name: String,
    pub value: Option<String>,
    pub severity: Severity,
    pub confidence: f32,
    pub evidence: Option<String>,
}

impl Finding {
    pub fn technology(
        name: impl Into<String>,
        category: impl Into<String>,
        confidence: f32,
        evidence: impl Into<String>,
    ) -> Self {
        Self {
            kind: FindingKind::Technology,
            name: name.into(),
            value: Some(category.into()),
            severity: Severity::Info,
            confidence: clamp_confidence(confidence),
            evidence: Some(evidence.into()),
        }
    }

    pub fn auth(
        name: impl Into<String>,
        value: Option<String>,
        confidence: f32,
        evidence: impl Into<String>,
    ) -> Self {
        Self {
            kind: FindingKind::Auth,
            name: name.into(),
            value,
            severity: Severity::Info,
            confidence: clamp_confidence(confidence),
            evidence: Some(evidence.into()),
        }
    }

    pub fn identity(&self) -> String {
        format!(
            "{}:{}:{}",
            self.kind.as_str(),
            self.name,
            self.value.as_deref().unwrap_or_default()
        )
    }
}

pub fn clamp_confidence(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}
