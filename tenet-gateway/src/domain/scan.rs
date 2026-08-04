use chrono::{DateTime, Utc};
use tenet_types::{Engine, TargetKind};

#[derive(Debug, Clone)]
pub struct NewScan {
    pub id: String,
    pub target: String,
    pub kind: TargetKind,
    pub engine: Engine,
    pub max_scripts: i16,
    pub max_attempts: i16,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ScanRecord {
    pub id: String,
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
