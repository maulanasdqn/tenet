use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRequest, Request};
use axum::Json;
use garde::Validate;
use tenet_errors::AppError;

pub struct ValidatedJson<T>(pub T);

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: Validate<Context = ()>,
    Json<T>: FromRequest<S, Rejection = JsonRejection>,
{
    type Rejection = AppError;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(payload) = Json::<T>::from_request(request, state)
            .await
            .map_err(body_error)?;
        payload.validate()?;
        Ok(Self(payload))
    }
}

fn body_error(rejection: JsonRejection) -> AppError {
    AppError::ValidationError(rejection.body_text())
}
