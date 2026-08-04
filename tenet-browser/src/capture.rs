use std::sync::Arc;

use chromiumoxide::cdp::browser_protocol::network::{
    EventRequestWillBeSent, EventResponseReceived, ResourceType,
};
use chromiumoxide::Page;
use futures_util::StreamExt;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::headers::{api_key_name, as_pairs, header_value, label};

const MAX_STATUS: i64 = 599;

#[derive(Debug, Clone)]
pub struct CapturedRequest {
    pub method: String,
    pub url: String,
    pub resource_type: String,
    pub status: Option<u16>,
    pub authorization: Option<String>,
    pub api_key_header: Option<String>,
}

impl CapturedRequest {
    pub fn is_api_call(&self) -> bool {
        matches!(self.resource_type.as_str(), "xhr" | "fetch")
    }

    pub fn is_gated(&self) -> bool {
        matches!(self.status, Some(401 | 403 | 429))
    }

    pub fn auth_scheme(&self) -> Option<String> {
        let value = self.authorization.as_ref()?;
        let token = value.split_whitespace().next()?;
        Some(token.to_lowercase())
    }
}

#[derive(Debug, Clone, Default)]
pub struct MainResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
}

struct Pending {
    request_id: String,
    request: CapturedRequest,
}

pub struct NetworkCapture {
    pending: Arc<Mutex<Vec<Pending>>>,
    main: Arc<Mutex<Option<MainResponse>>>,
    tasks: Vec<JoinHandle<()>>,
}

impl NetworkCapture {
    pub async fn watch(page: &Page) -> Option<Self> {
        let mut sent = page.event_listener::<EventRequestWillBeSent>().await.ok()?;
        let mut received = page.event_listener::<EventResponseReceived>().await.ok()?;

        let pending = Arc::new(Mutex::new(Vec::new()));
        let main = Arc::new(Mutex::new(None));
        let sent_sink = Arc::clone(&pending);
        let received_pending = Arc::clone(&pending);
        let received_main = Arc::clone(&main);

        let tasks = vec![
            tokio::spawn(async move {
                while let Some(event) = sent.next().await {
                    record_request(&event, &sent_sink).await;
                }
            }),
            tokio::spawn(async move {
                while let Some(event) = received.next().await {
                    record_response(&event, &received_pending, &received_main).await;
                }
            }),
        ];

        Some(Self {
            pending,
            main,
            tasks,
        })
    }

    pub async fn requests(&self) -> Vec<CapturedRequest> {
        self.pending
            .lock()
            .await
            .iter()
            .map(|entry| entry.request.clone())
            .collect()
    }

    pub async fn main_response(&self) -> Option<MainResponse> {
        self.main.lock().await.clone()
    }
}

impl Drop for NetworkCapture {
    fn drop(&mut self) {
        for task in &self.tasks {
            task.abort();
        }
    }
}

async fn record_request(event: &EventRequestWillBeSent, sink: &Arc<Mutex<Vec<Pending>>>) {
    sink.lock().await.push(Pending {
        request_id: event.request_id.inner().clone(),
        request: as_captured(event),
    });
}

async fn record_response(
    event: &EventResponseReceived,
    pending: &Arc<Mutex<Vec<Pending>>>,
    main: &Arc<Mutex<Option<MainResponse>>>,
) {
    let status = event.response.status.clamp(0, MAX_STATUS) as u16;
    let request_id = event.request_id.inner().clone();
    attach_status(pending, &request_id, status).await;
    if event.r#type == ResourceType::Document {
        *main.lock().await = Some(MainResponse {
            status,
            headers: as_pairs(&event.response.headers),
        });
    }
}

async fn attach_status(pending: &Arc<Mutex<Vec<Pending>>>, request_id: &str, status: u16) {
    let mut guard = pending.lock().await;
    if let Some(entry) = guard
        .iter_mut()
        .find(|entry| entry.request_id == request_id)
    {
        entry.request.status = Some(status);
    }
}

fn as_captured(event: &EventRequestWillBeSent) -> CapturedRequest {
    let headers = as_pairs(&event.request.headers);
    CapturedRequest {
        method: event.request.method.to_lowercase(),
        url: event.request.url.clone(),
        resource_type: label(event.r#type.as_ref()),
        status: None,
        authorization: header_value(&headers, "authorization"),
        api_key_header: api_key_name(&headers),
    }
}

#[cfg(test)]
mod tests {
    use super::CapturedRequest;

    fn request(resource_type: &str, status: Option<u16>) -> CapturedRequest {
        CapturedRequest {
            method: "get".to_owned(),
            url: "https://example.com/api/v1/me".to_owned(),
            resource_type: resource_type.to_owned(),
            status,
            authorization: Some("Bearer abc.def".to_owned()),
            api_key_header: None,
        }
    }

    #[test]
    fn an_xhr_is_an_api_call_but_a_script_is_not() {
        assert!(request("xhr", None).is_api_call());
        assert!(request("fetch", None).is_api_call());
        assert!(!request("script", None).is_api_call());
    }

    #[test]
    fn the_scheme_is_the_first_token_of_the_header() {
        assert_eq!(
            request("xhr", None).auth_scheme(),
            Some("bearer".to_owned())
        );
    }

    #[test]
    fn a_forbidden_or_rate_limited_response_is_gated() {
        assert!(request("xhr", Some(403)).is_gated());
        assert!(request("xhr", Some(401)).is_gated());
        assert!(request("xhr", Some(429)).is_gated());
    }

    #[test]
    fn a_healthy_or_unknown_response_is_not_gated() {
        assert!(!request("xhr", Some(200)).is_gated());
        assert!(!request("xhr", None).is_gated());
    }
}
