use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Model not loaded: {0}")]
    ModelNotLoaded(String),

    #[error("Generation failed: {0}")]
    GenerationFailed(String),

    #[error("Tokenization error: {0}")]
    TokenizationError(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            AppError::ModelNotLoaded(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg.clone()),
            AppError::GenerationFailed(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            AppError::TokenizationError(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::InvalidRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
