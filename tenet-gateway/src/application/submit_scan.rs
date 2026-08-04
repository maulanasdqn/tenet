use std::sync::Arc;

use chrono::Utc;
use tenet_errors::AppError;
use tenet_types::{Engine, TargetKind};
use ulid::Ulid;

use crate::domain::ports::ScanRepository;
use crate::domain::scan::NewScan;

const MAX_SCRIPTS_CEILING: i16 = 100;

pub struct SubmitScanInput {
    pub target: String,
    pub kind: TargetKind,
    pub engine: Engine,
    pub max_scripts: Option<i16>,
}

pub struct SubmitScan {
    repository: Arc<dyn ScanRepository>,
    default_max_scripts: i16,
    max_attempts: i16,
}

impl SubmitScan {
    pub fn new(
        repository: Arc<dyn ScanRepository>,
        default_max_scripts: i16,
        max_attempts: i16,
    ) -> Self {
        Self {
            repository,
            default_max_scripts,
            max_attempts,
        }
    }

    pub async fn execute(&self, input: SubmitScanInput) -> Result<NewScan, AppError> {
        let target = validated_target(&input)?;
        let scan = NewScan {
            id: Ulid::new().to_string(),
            target,
            kind: input.kind,
            engine: engine_for(input.kind, input.engine),
            max_scripts: clamp_scripts(input.max_scripts.unwrap_or(self.default_max_scripts)),
            max_attempts: self.max_attempts,
            created_at: Utc::now(),
        };
        self.repository.insert(&scan).await?;
        Ok(scan)
    }
}

fn validated_target(input: &SubmitScanInput) -> Result<String, AppError> {
    match input.kind {
        TargetKind::Web => validated_url(&input.target),
        TargetKind::Mobile => validated_reference(&input.target),
    }
}

fn validated_url(raw: &str) -> Result<String, AppError> {
    let parsed = url::Url::parse(raw)
        .map_err(|_| AppError::ValidationError("target is not a valid url".to_owned()))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(AppError::ValidationError(
            "target must be an http or https url".to_owned(),
        ));
    }
    if parsed.host_str().is_none() {
        return Err(AppError::ValidationError(
            "target must name a host".to_owned(),
        ));
    }
    Ok(parsed.to_string())
}

fn validated_reference(raw: &str) -> Result<String, AppError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AppError::ValidationError(
            "target must reference a binary or package".to_owned(),
        ));
    }
    Ok(trimmed.to_owned())
}

fn clamp_scripts(requested: i16) -> i16 {
    requested.clamp(1, MAX_SCRIPTS_CEILING)
}

fn engine_for(kind: TargetKind, requested: Engine) -> Engine {
    match kind {
        TargetKind::Mobile => Engine::Http,
        TargetKind::Web => requested,
    }
}

#[cfg(test)]
mod tests {
    use tenet_types::{Engine, TargetKind};

    use super::{clamp_scripts, engine_for, validated_target, SubmitScanInput};

    fn input(target: &str, kind: TargetKind) -> SubmitScanInput {
        SubmitScanInput {
            target: target.to_owned(),
            kind,
            engine: Engine::Http,
            max_scripts: None,
        }
    }

    #[test]
    fn a_mobile_target_never_uses_the_browser_engine() {
        assert_eq!(
            engine_for(TargetKind::Mobile, Engine::Browser),
            Engine::Http
        );
        assert_eq!(
            engine_for(TargetKind::Web, Engine::Browser),
            Engine::Browser
        );
    }

    #[test]
    fn a_web_target_must_be_an_http_url() {
        assert!(validated_target(&input("https://example.com", TargetKind::Web)).is_ok());
        assert!(validated_target(&input("example.com", TargetKind::Web)).is_err());
        assert!(validated_target(&input("ftp://example.com", TargetKind::Web)).is_err());
    }

    #[test]
    fn a_mobile_target_only_has_to_be_present() {
        assert!(validated_target(&input("com.example.app", TargetKind::Mobile)).is_ok());
        assert!(validated_target(&input("   ", TargetKind::Mobile)).is_err());
    }

    #[test]
    fn the_script_budget_stays_inside_the_ceiling() {
        assert_eq!(clamp_scripts(0), 1);
        assert_eq!(clamp_scripts(25), 25);
        assert_eq!(clamp_scripts(5000), 100);
    }
}
