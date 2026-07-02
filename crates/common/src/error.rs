use thiserror::Error;
use axum::{
    response::{IntoResponse, Response},
    http::StatusCode,
    Json,
};
use serde_json::json;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Internal server error: {0}")]
    Internal(#[from] anyhow::Error),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Not found: {0}")]
    NotFound(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Internal(e) => {
                tracing::error!("Internal error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
        };

        let body = Json(json!({
            "error": message
        }));

        (status, body).into_response()
    }
}