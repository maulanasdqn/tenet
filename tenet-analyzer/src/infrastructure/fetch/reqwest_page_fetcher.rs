use std::time::Duration;

use reqwest::redirect::Policy;
use reqwest::Client;
use sha2::{Digest, Sha256};
use tenet_errors::AppError;

use crate::domain::ports::PageFetcher;
use crate::domain::work::FetchedDocument;

const SERVER_ERROR_FLOOR: u16 = 500;

pub struct FetchSettings {
    pub timeout_seconds: u64,
    pub connect_timeout_seconds: u64,
    pub max_redirects: usize,
    pub user_agent: String,
    pub max_body_bytes: usize,
}

pub struct ReqwestPageFetcher {
    client: Client,
    max_body_bytes: usize,
}

impl ReqwestPageFetcher {
    pub fn new(settings: FetchSettings) -> Result<Self, AppError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(settings.timeout_seconds))
            .connect_timeout(Duration::from_secs(settings.connect_timeout_seconds))
            .redirect(Policy::limited(settings.max_redirects))
            .user_agent(settings.user_agent)
            .build()?;
        Ok(Self {
            client,
            max_body_bytes: settings.max_body_bytes,
        })
    }
}

#[async_trait::async_trait]
impl PageFetcher for ReqwestPageFetcher {
    async fn fetch(&self, url: &str) -> Result<FetchedDocument, AppError> {
        let response = self.client.get(url).send().await?;
        let status = response.status().as_u16();
        let final_url = response.url().to_string();
        let headers = response
            .headers()
            .iter()
            .filter_map(|(name, value)| {
                value
                    .to_str()
                    .ok()
                    .map(|text| (name.as_str().to_owned(), text.to_owned()))
            })
            .collect();

        if status >= SERVER_ERROR_FLOOR {
            return Err(AppError::Unavailable(format!(
                "{final_url} answered with {status}"
            )));
        }

        let bytes = response.bytes().await?;
        let kept = &bytes[..bytes.len().min(self.max_body_bytes)];
        Ok(FetchedDocument {
            url: final_url,
            status,
            headers,
            body: String::from_utf8_lossy(kept).into_owned(),
            sha256: hex::encode(Sha256::digest(kept)),
            byte_size: bytes.len() as i64,
        })
    }

    async fn fetch_bytes(&self, url: &str) -> Result<Vec<u8>, AppError> {
        let response = self.client.get(url).send().await?;
        let status = response.status().as_u16();
        if status >= SERVER_ERROR_FLOOR {
            return Err(AppError::Unavailable(format!(
                "{url} answered with {status}"
            )));
        }
        let bytes = response.bytes().await?;
        let kept = &bytes[..bytes.len().min(self.max_body_bytes)];
        Ok(kept.to_vec())
    }
}
