use axum::extract::State;
use axum::Json;
use common::{HelloWorldRequest, HelloWorldResponse, HealthResponse, AppError};
use crate::routes::AppState;
use metrics::counter;
use std::time::Instant;

pub async fn hello_world() -> Json<HelloWorldResponse> {
    counter!("hello_world_requests_total").increment(1);
    Json(HelloWorldResponse {
        message: "Hello, World!".to_string(),
    })
}

pub async fn hello_world_post(
    State(_state): State<AppState>,
    Json(req): Json<HelloWorldRequest>,
) -> Result<Json<HelloWorldResponse>, AppError> {
    let start = Instant::now();
    let name = req.name.unwrap_or_else(|| "World".to_string());
    let response = HelloWorldResponse {
        message: format!("Hello, {}!", name),
    };
    counter!("hello_world_post_requests_total").increment(1);
    metrics::histogram!("hello_world_post_duration_seconds").record(start.elapsed().as_secs_f64());
    Ok(Json(response))
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
    })
}