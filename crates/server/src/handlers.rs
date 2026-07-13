use axum::extract::State;
use axum::Json;
use common::{HelloWorldRequest, HelloWorldResponse, HealthResponse};
use crate::error::ApiError;
use crate::routes::AppState;
use metrics::counter;
use std::time::Instant;

#[utoipa::path(
    get,
    path = "/",
    tag = "hello",
    responses(
        (status = 200, description = "Returns a hello world message", body = HelloWorldResponse)
    )
)]
pub async fn hello_world() -> Json<HelloWorldResponse> {
    counter!("hello_world_requests_total").increment(1);
    Json(HelloWorldResponse {
        message: "Hello, World!".to_string(),
    })
}

#[utoipa::path(
    post,
    path = "/",
    tag = "hello",
    request_body = HelloWorldRequest,
    responses(
        (status = 200, description = "Returns a personalized hello message", body = HelloWorldResponse),
        (status = 400, description = "Bad request", body = String)
    )
)]
pub async fn hello_world_post(
    State(_state): State<AppState>,
    Json(req): Json<HelloWorldRequest>,
) -> Result<Json<HelloWorldResponse>, ApiError> {
    let start = Instant::now();
    let name = req.name.unwrap_or_else(|| "World".to_string());
    let response = HelloWorldResponse {
        message: format!("Hello, {}!", name),
    };
    counter!("hello_world_post_requests_total").increment(1);
    metrics::histogram!("hello_world_post_duration_seconds").record(start.elapsed().as_secs_f64());
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/health",
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