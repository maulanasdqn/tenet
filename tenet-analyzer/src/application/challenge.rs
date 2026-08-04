use tenet_types::{Finding, FindingKind, Severity};

use crate::domain::work::FetchedDocument;

const CHALLENGE_CONFIDENCE: f32 = 0.9;

pub fn challenge_findings(document: &FetchedDocument) -> Vec<Finding> {
    let Some(challenge) = tenet_stealth::detect_challenge(&document.url, &document.body) else {
        return Vec::new();
    };
    tracing::warn!(
        url = %document.url,
        vendor = %challenge.vendor,
        "the target served a bot challenge, results may be incomplete"
    );
    vec![Finding {
        kind: FindingKind::Technology,
        name: challenge.vendor.to_owned(),
        value: Some("bot-protection".to_owned()),
        severity: Severity::Medium,
        confidence: CHALLENGE_CONFIDENCE,
        evidence: Some(challenge.evidence),
    }]
}

#[cfg(test)]
mod tests {
    use crate::domain::work::FetchedDocument;

    use super::challenge_findings;

    fn document(url: &str, body: &str) -> FetchedDocument {
        FetchedDocument {
            url: url.to_owned(),
            status: 200,
            headers: Vec::new(),
            body: body.to_owned(),
            sha256: String::new(),
            byte_size: 0,
        }
    }

    #[test]
    fn a_challenge_page_becomes_a_protection_finding() {
        let found = challenge_findings(&document(
            "https://shopee.co.id/verify/traffic/error",
            "<html>",
        ));
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].value, Some("bot-protection".to_owned()));
        assert_eq!(found[0].name, "Shopee traffic verification");
    }

    #[test]
    fn an_ordinary_page_produces_no_protection_finding() {
        assert!(
            challenge_findings(&document("https://example.com/", "<html>hi</html>")).is_empty()
        );
    }
}
