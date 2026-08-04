use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use crate::domain::artifact::{EndpointRecord, FindingRecord};
use crate::domain::scan::ScanRecord;

#[derive(Debug, Serialize, ToSchema)]
pub struct ScanResponse {
    pub scan_id: String,
    pub target: String,
    pub kind: String,
    pub engine: String,
    pub status: String,
    pub attempts: i16,
    pub max_attempts: i16,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
}

impl From<ScanRecord> for ScanResponse {
    fn from(record: ScanRecord) -> Self {
        Self {
            scan_id: record.id,
            target: record.target,
            kind: record.kind,
            engine: record.engine,
            status: record.status,
            attempts: record.attempts,
            max_attempts: record.max_attempts,
            error: record.error,
            created_at: record.created_at,
            started_at: record.started_at,
            finished_at: record.finished_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FindingResponse {
    pub kind: String,
    pub name: String,
    pub value: Option<String>,
    pub severity: String,
    pub confidence: f32,
    pub evidence: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<FindingRecord> for FindingResponse {
    fn from(record: FindingRecord) -> Self {
        Self {
            kind: record.kind,
            name: record.name,
            value: record.value,
            severity: record.severity,
            confidence: record.confidence,
            evidence: record.evidence,
            created_at: record.created_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EndpointResponse {
    pub method: String,
    pub path: String,
    pub base_url: Option<String>,
    pub source: String,
    pub confidence: f32,
    pub created_at: DateTime<Utc>,
}

impl From<EndpointRecord> for EndpointResponse {
    fn from(record: EndpointRecord) -> Self {
        Self {
            method: record.method,
            path: record.path,
            base_url: record.base_url,
            source: record.source,
            confidence: record.confidence,
            created_at: record.created_at,
        }
    }
}
