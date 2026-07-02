use axum::extract::State;
use axum::Json;
use common::{HelloWorldRequest, HelloWorldResponse, HealthResponse, AppError};
use crate::routes::AppState;

pub async fn hello_world() -> Json<HelloWorldResponse> {
    Json(HelloWorldResponse {
        message: "Hello, World!".to_string(),
    })
}

pub async fn hello_world_post(
    State(_state): State<AppState>,
    Json(req): Json<HelloWorldRequest>,
) -> Result<Json<HelloWorldResponse>, AppError> {
    let name = req.name.unwrap_or_else(|| "World".to_string());
    Ok(Json(HelloWorldResponse {
        message: format!("Hello, {}!", name),
    }))
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
    })
}