use std::path::Path;

use tenet_errors::AppError;
use tenet_types::{Endpoint, Finding};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryFormat {
    Apk,
    Ipa,
}

impl BinaryFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            BinaryFormat::Apk => "apk",
            BinaryFormat::Ipa => "ipa",
        }
    }

    pub fn from_reference(reference: &str) -> Option<Self> {
        let extension = Path::new(reference).extension()?.to_str()?;
        if extension.eq_ignore_ascii_case("apk") {
            return Some(BinaryFormat::Apk);
        }
        if extension.eq_ignore_ascii_case("ipa") {
            return Some(BinaryFormat::Ipa);
        }
        None
    }
}

#[derive(Debug, Clone, Default)]
pub struct BinaryReport {
    pub package: String,
    pub findings: Vec<Finding>,
    pub endpoints: Vec<Endpoint>,
}

#[async_trait::async_trait]
pub trait BinaryAnalyzer: Send + Sync {
    async fn analyze(&self, reference: &str) -> Result<BinaryReport, AppError>;
}

pub struct PendingBinaryAnalyzer;

#[async_trait::async_trait]
impl BinaryAnalyzer for PendingBinaryAnalyzer {
    async fn analyze(&self, reference: &str) -> Result<BinaryReport, AppError> {
        let format =
            BinaryFormat::from_reference(reference).map_or("unknown", |binary| binary.as_str());
        Err(AppError::Unavailable(format!(
            "mobile binary analysis is not available yet for {format} targets"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::{BinaryAnalyzer, BinaryFormat, PendingBinaryAnalyzer};

    #[test]
    fn an_apk_reference_is_recognised() {
        assert_eq!(
            BinaryFormat::from_reference("/tmp/App-release.APK"),
            Some(BinaryFormat::Apk)
        );
    }

    #[test]
    fn an_unknown_reference_has_no_format() {
        assert_eq!(BinaryFormat::from_reference("com.example.app"), None);
    }

    #[tokio::test]
    async fn the_pending_analyzer_reports_that_it_is_unavailable() {
        let result = PendingBinaryAnalyzer.analyze("app.ipa").await;
        assert!(result.is_err());
    }
}
