use crate::{AppError, FieldError};

impl From<garde::Report> for AppError {
    fn from(report: garde::Report) -> Self {
        AppError::Invalid(
            report
                .iter()
                .map(|(path, error)| FieldError {
                    field: path.to_string(),
                    message: error.to_string(),
                })
                .collect(),
        )
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AppError::NotFound("record not found".to_owned()),
            sqlx::Error::Database(db) if db.is_unique_violation() => {
                AppError::Conflict(db.to_string())
            }
            sqlx::Error::Io(_)
            | sqlx::Error::PoolTimedOut
            | sqlx::Error::PoolClosed
            | sqlx::Error::Tls(_)
            | sqlx::Error::Protocol(_)
            | sqlx::Error::WorkerCrashed => AppError::unavailable(err),
            other => AppError::internal(other),
        }
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() || err.is_connect() || err.is_request() {
            return AppError::unavailable(err);
        }
        AppError::internal(err)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::bad_request(err)
    }
}
