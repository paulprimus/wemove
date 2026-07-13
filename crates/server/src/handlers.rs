use axum::extract::State;
use axum::Json;
use common::{MainRequest, MainResponse, HealthResponse};
use crate::error::ApiError;
use crate::routes::AppState;
use metrics::counter;
use std::time::Instant;

#[utoipa::path(
    get,
    path = "/api/main",
    tag = "main",
    responses(
        (status = 200, description = "Returns a greeting message", body = MainResponse)
    )
)]
pub async fn main_get() -> Json<MainResponse> {
    tracing::debug!("main_get called");
    counter!("main_get_requests_total").increment(1);
    Json(MainResponse {
        message: "Hello, WeMove!".to_string(),
    })
}

#[utoipa::path(
    post,
    path = "/api/main",
    tag = "main",
    request_body = MainRequest,
    responses(
        (status = 200, description = "Returns a personalized greeting message", body = MainResponse),
        (status = 400, description = "Bad request", body = String)
    )
)]
pub async fn main_post(
    State(_state): State<AppState>,
    Json(req): Json<MainRequest>,
) -> Result<Json<MainResponse>, ApiError> {
    let start = Instant::now();
    let name = req.name.unwrap_or_else(|| "World".to_string());
    let response = MainResponse {
        message: format!("Hello, {}!", name),
    };
    counter!("main_post_requests_total").increment(1);
    metrics::histogram!("main_post_duration_seconds").record(start.elapsed().as_secs_f64());
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/api/health",
    tag = "health",
    responses(
        (status = 200, description = "Returns the health status", body = HealthResponse)
    )
)]
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
    })
}