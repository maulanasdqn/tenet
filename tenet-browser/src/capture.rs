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
    pub authorization: Option<String>,
    pub api_key_header: Option<String>,
}

impl CapturedRequest {
    pub fn is_api_call(&self) -> bool {
        matches!(self.resource_type.as_str(), "xhr" | "fetch")
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

pub struct NetworkCapture {
    requests: Arc<Mutex<Vec<CapturedRequest>>>,
    main: Arc<Mutex<Option<MainResponse>>>,
    tasks: Vec<JoinHandle<()>>,
}

impl NetworkCapture {
    pub async fn watch(page: &Page) -> Option<Self> {
        let mut sent = page.event_listener::<EventRequestWillBeSent>().await.ok()?;
        let mut received = page.event_listener::<EventResponseReceived>().await.ok()?;

        let requests = Arc::new(Mutex::new(Vec::new()));
        let main = Arc::new(Mutex::new(None));
        let request_sink = Arc::clone(&requests);
        let main_sink = Arc::clone(&main);

        let tasks = vec![
            tokio::spawn(async move {
                while let Some(event) = sent.next().await {
                    request_sink.lock().await.push(as_captured(&event));
                }
            }),
            tokio::spawn(async move {
                while let Some(event) = received.next().await {
                    store_main_response(&event, &main_sink).await;
                }
            }),
        ];

        Some(Self {
            requests,
            main,
            tasks,
        })
    }

    pub async fn requests(&self) -> Vec<CapturedRequest> {
        self.requests.lock().await.clone()
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

async fn store_main_response(
    event: &EventResponseReceived,
    sink: &Arc<Mutex<Option<MainResponse>>>,
) {
    if event.r#type != ResourceType::Document {
        return;
    }
    *sink.lock().await = Some(MainResponse {
        status: event.response.status.clamp(0, MAX_STATUS) as u16,
        headers: as_pairs(&event.response.headers),
    });
}

fn as_captured(event: &EventRequestWillBeSent) -> CapturedRequest {
    let headers = as_pairs(&event.request.headers);
    CapturedRequest {
        method: event.request.method.to_lowercase(),
        url: event.request.url.clone(),
        resource_type: label(event.r#type.as_ref()),
        authorization: header_value(&headers, "authorization"),
        api_key_header: api_key_name(&headers),
    }
}

#[cfg(test)]
mod tests {
    use super::CapturedRequest;

    fn request(resource_type: &str, authorization: Option<&str>) -> CapturedRequest {
        CapturedRequest {
            method: "get".to_owned(),
            url: "https://example.com/api/v1/me".to_owned(),
            resource_type: resource_type.to_owned(),
            authorization: authorization.map(ToOwned::to_owned),
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
        let scheme = request("xhr", Some("Bearer abc.def")).auth_scheme();
        assert_eq!(scheme, Some("bearer".to_owned()));
    }

    #[test]
    fn a_request_without_authorization_has_no_scheme() {
        assert_eq!(request("xhr", None).auth_scheme(), None);
    }
}
