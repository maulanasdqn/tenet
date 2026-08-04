use std::collections::HashMap;
use std::sync::Arc;

use tenet_errors::AppError;
use tenet_types::{Endpoint, Finding};
use tenet_web::{PageSnapshot, ScriptAsset};

use crate::domain::ports::{PageFetcher, TargetAnalyzer};
use crate::domain::work::{Analysis, ArtifactRecord, FetchedDocument, ScanClaim};

pub struct AnalyzeWebTarget {
    fetcher: Arc<dyn PageFetcher>,
}

impl AnalyzeWebTarget {
    pub fn new(fetcher: Arc<dyn PageFetcher>) -> Self {
        Self { fetcher }
    }

    async fn harvest(
        &self,
        document: &FetchedDocument,
        budget: usize,
        artifacts: &mut Vec<ArtifactRecord>,
    ) -> Vec<ScriptAsset> {
        let mut scripts = Vec::new();
        for url in tenet_web::script_urls(&document.body, &document.url)
            .into_iter()
            .take(budget)
        {
            match self.fetcher.fetch(&url).await {
                Ok(script) => {
                    artifacts.push(ArtifactRecord::from_document(&script, "script"));
                    scripts.push(ScriptAsset::new(script.url, script.body));
                }
                Err(err) => tracing::warn!(url = %url, error = %err, "script fetch failed"),
            }
        }
        for (index, body) in tenet_web::inline_scripts(&document.body)
            .into_iter()
            .enumerate()
        {
            scripts.push(ScriptAsset::new(
                format!("{}#inline-{index}", document.url),
                body,
            ));
        }
        scripts
    }
}

#[async_trait::async_trait]
impl TargetAnalyzer for AnalyzeWebTarget {
    async fn analyze(&self, claim: &ScanClaim) -> Result<Analysis, AppError> {
        let document = self.fetcher.fetch(&claim.target).await?;
        let mut artifacts = vec![ArtifactRecord::from_document(&document, "html")];
        let scripts = self
            .harvest(&document, claim.max_scripts.max(0) as usize, &mut artifacts)
            .await;

        let page = PageSnapshot::new(
            document.url.clone(),
            document.status,
            document.headers.clone(),
            document.body.clone(),
        );
        let endpoints = collect_endpoints(&document, &scripts);
        let mut raw = tenet_web::detect(&page, &scripts);
        raw.extend(tenet_web::auth_findings(&page, &scripts, &endpoints));
        let findings = dedupe_findings(raw);

        tracing::info!(
            scan_id = %claim.id,
            artifacts = artifacts.len(),
            endpoints = endpoints.len(),
            findings = findings.len(),
            "web target analyzed"
        );

        Ok(Analysis {
            artifacts,
            findings,
            endpoints,
        })
    }
}

fn collect_endpoints(document: &FetchedDocument, scripts: &[ScriptAsset]) -> Vec<Endpoint> {
    let mut found = tenet_web::endpoints(&document.body, &document.url);
    for script in scripts {
        found.extend(tenet_web::endpoints(&script.body, &script.url));
    }
    dedupe(found)
}

fn dedupe(found: Vec<Endpoint>) -> Vec<Endpoint> {
    let mut best: HashMap<String, Endpoint> = HashMap::new();
    for endpoint in found {
        let identity = endpoint.identity();
        let keep = best
            .get(&identity)
            .is_none_or(|existing| endpoint.confidence > existing.confidence);
        if keep {
            best.insert(identity, endpoint);
        }
    }
    let mut result: Vec<Endpoint> = best.into_values().collect();
    result.sort_by_key(Endpoint::identity);
    result
}

pub fn dedupe_findings(found: Vec<Finding>) -> Vec<Finding> {
    let mut best: HashMap<String, Finding> = HashMap::new();
    for finding in found {
        let identity = finding.identity();
        let keep = best
            .get(&identity)
            .is_none_or(|existing| finding.confidence > existing.confidence);
        if keep {
            best.insert(identity, finding);
        }
    }
    best.into_values().collect()
}

#[cfg(test)]
mod tests {
    use tenet_types::{Endpoint, HttpMethod};

    use super::dedupe;

    #[test]
    fn a_repeated_endpoint_keeps_the_strongest_signal() {
        let found = vec![
            Endpoint::new(HttpMethod::Get, "/api/x", "page", 0.4),
            Endpoint::new(HttpMethod::Get, "/api/x", "bundle.js", 0.9),
        ];
        let deduped = dedupe(found);
        assert_eq!(deduped.len(), 1);
        assert_eq!(deduped[0].source, "bundle.js");
    }

    #[test]
    fn distinct_verbs_stay_apart() {
        let found = vec![
            Endpoint::new(HttpMethod::Get, "/api/x", "page", 0.4),
            Endpoint::new(HttpMethod::Post, "/api/x", "page", 0.4),
        ];
        assert_eq!(dedupe(found).len(), 2);
    }
}
