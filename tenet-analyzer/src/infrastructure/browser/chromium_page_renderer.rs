use sha2::{Digest, Sha256};
use tenet_browser::{CapturedRequest, ChromiumRenderer, RenderSettings};
use tenet_errors::AppError;

use crate::domain::ports::PageRenderer;
use crate::domain::work::{FetchedDocument, ObservedRequest, RenderedDocument};

pub struct ChromiumPageRenderer {
    renderer: ChromiumRenderer,
}

impl ChromiumPageRenderer {
    pub async fn open(settings: RenderSettings, pool_size: usize) -> Result<Self, AppError> {
        Ok(Self {
            renderer: ChromiumRenderer::open(settings, pool_size).await?,
        })
    }
}

#[async_trait::async_trait]
impl PageRenderer for ChromiumPageRenderer {
    async fn render(&self, url: &str) -> Result<RenderedDocument, AppError> {
        let page = self.renderer.render(url).await?;
        let sha256 = hex::encode(Sha256::digest(page.html.as_bytes()));
        let byte_size = page.html.len() as i64;

        Ok(RenderedDocument {
            document: FetchedDocument {
                url: page.url,
                status: page.status,
                headers: page.headers,
                body: page.html,
                sha256,
                byte_size,
            },
            observed: page
                .captured
                .iter()
                .filter(|request| request.is_api_call())
                .map(as_observed)
                .collect(),
            storage_keys: page.storage_keys,
            wasm_modules: wasm_modules(&page.captured),
        })
    }
}

fn as_observed(request: &CapturedRequest) -> ObservedRequest {
    ObservedRequest {
        method: request.method.clone(),
        url: request.url.clone(),
        status: request.status,
        gated: request.is_gated(),
        auth_scheme: request.auth_scheme(),
        api_key_header: request.api_key_header.clone(),
    }
}

fn wasm_modules(captured: &[CapturedRequest]) -> Vec<String> {
    let mut modules: Vec<String> = captured
        .iter()
        .map(|request| request.url.split(['?', '#']).next().unwrap_or(&request.url))
        .filter(|url| url.to_lowercase().ends_with(".wasm"))
        .map(ToOwned::to_owned)
        .collect();
    modules.sort();
    modules.dedup();
    modules
}
