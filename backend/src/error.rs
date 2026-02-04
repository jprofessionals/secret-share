use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Crypto error: {0}")]
    CryptoError(String),

    #[error("Secret not found")]
    NotFound,

    #[error("Secret has expired")]
    Expired,

    #[error("Maximum views reached")]
    MaxViewsReached,

    #[error("Invalid passphrase")]
    InvalidPassphrase,

    #[error("Bad request")]
    BadRequest,

    #[error("Internal server error")]
    InternalError,

    #[error("Secret cannot be extended")]
    NotExtendable,

    #[error("Extension exceeds maximum limits")]
    ExceedsLimits,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::CryptoError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::Expired => (StatusCode::GONE, self.to_string()),
            AppError::MaxViewsReached => (StatusCode::GONE, self.to_string()),
            AppError::InvalidPassphrase => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::BadRequest => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::NotExtendable => (StatusCode::FORBIDDEN, self.to_string()),
            AppError::ExceedsLimits => (StatusCode::BAD_REQUEST, self.to_string()),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

impl From<bip39::Error> for AppError {
    fn from(err: bip39::Error) -> Self {
        AppError::CryptoError(err.to_string())
    }
}
