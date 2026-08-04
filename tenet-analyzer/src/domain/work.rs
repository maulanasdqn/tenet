use tenet_types::{Endpoint, Finding};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ScanClaim {
    pub id: String,
    pub target: String,
    pub kind: String,
    pub engine: String,
    pub max_scripts: i16,
    pub attempts: i16,
    pub max_attempts: i16,
}

#[derive(Debug, Clone)]
pub struct FetchedDocument {
    pub url: String,
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub sha256: String,
    pub byte_size: i64,
}

#[derive(Debug, Clone)]
pub struct ObservedRequest {
    pub method: String,
    pub url: String,
    pub status: Option<u16>,
    pub gated: bool,
    pub auth_scheme: Option<String>,
    pub api_key_header: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RenderedDocument {
    pub document: FetchedDocument,
    pub observed: Vec<ObservedRequest>,
    pub storage_keys: Vec<String>,
    pub wasm_modules: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ArtifactRecord {
    pub url: String,
    pub kind: String,
    pub sha256: String,
    pub byte_size: i64,
}

impl ArtifactRecord {
    pub fn from_document(document: &FetchedDocument, kind: &str) -> Self {
        Self {
            url: document.url.clone(),
            kind: kind.to_owned(),
            sha256: document.sha256.clone(),
            byte_size: document.byte_size,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Analysis {
    pub artifacts: Vec<ArtifactRecord>,
    pub findings: Vec<Finding>,
    pub endpoints: Vec<Endpoint>,
}
