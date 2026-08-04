use chrono::{DateTime, Utc};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FindingRecord {
    pub kind: String,
    pub name: String,
    pub value: Option<String>,
    pub severity: String,
    pub confidence: f32,
    pub evidence: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EndpointRecord {
    pub method: String,
    pub path: String,
    pub base_url: Option<String>,
    pub source: String,
    pub confidence: f32,
    pub created_at: DateTime<Utc>,
}
