use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use common::AppError;
use serde_json::json;

/// Newtype-Wrapper um `common::AppError`, der die HTTP-Übersetzung
/// (`IntoResponse`) im Web-Layer ansiedelt. So bleibt `common` frei von
/// einer Axum-Abhängigkeit (Orphan-Rule verhindert `impl IntoResponse for
/// AppError` direkt in `server`, da weder Trait noch Typ hier lokal sind).
pub struct ApiError(pub AppError);

impl From<AppError> for ApiError {
    fn from(err: AppError) -> Self {
        ApiError(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self.0 {
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, msg.clone())
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
