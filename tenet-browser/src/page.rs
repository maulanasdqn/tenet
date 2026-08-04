use crate::capture::CapturedRequest;

#[derive(Debug, Clone, Default)]
pub struct RenderedPage {
    pub url: String,
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub html: String,
    pub captured: Vec<CapturedRequest>,
    pub storage_keys: Vec<String>,
}

impl RenderedPage {
    pub fn xhr_requests(&self) -> Vec<&CapturedRequest> {
        self.captured
            .iter()
            .filter(|request| request.is_api_call())
            .collect()
    }
}
