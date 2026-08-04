use axum::extract::{Request, State};
use axum::http::header::AUTHORIZATION;
use axum::middleware::Next;
use axum::response::Response;
use tenet_errors::AppError;

use crate::state::AppState;

const API_KEY_HEADER: &str = "x-api-key";

pub async fn require_api_key(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let presented = presented_key(&request)
        .ok_or_else(|| AppError::Unauthorized("api key is missing".to_owned()))?;
    if presented != state.api_key.as_str() {
        return Err(AppError::Unauthorized("api key was rejected".to_owned()));
    }
    Ok(next.run(request).await)
}

fn presented_key(request: &Request) -> Option<&str> {
    let headers = request.headers();
    if let Some(key) = headers
        .get(API_KEY_HEADER)
        .and_then(|value| value.to_str().ok())
    {
        return Some(key);
    }
    headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
}
