use std::sync::Arc;

use tenet_web::ScriptAsset;

use crate::domain::ports::PageFetcher;
use crate::domain::work::ArtifactRecord;

pub async fn harvest_scripts(
    fetcher: &Arc<dyn PageFetcher>,
    html: &str,
    base_url: &str,
    budget: usize,
    artifacts: &mut Vec<ArtifactRecord>,
) -> Vec<ScriptAsset> {
    let mut scripts = Vec::new();
    for url in tenet_web::script_urls(html, base_url)
        .into_iter()
        .take(budget)
    {
        match fetcher.fetch(&url).await {
            Ok(script) => {
                artifacts.push(ArtifactRecord::from_document(&script, "script"));
                scripts.push(ScriptAsset::new(script.url, script.body));
            }
            Err(err) => tracing::warn!(url = %url, error = %err, "script fetch failed"),
        }
    }
    for (index, body) in tenet_web::inline_scripts(html).into_iter().enumerate() {
        scripts.push(ScriptAsset::new(format!("{base_url}#inline-{index}"), body));
    }
    scripts
}
