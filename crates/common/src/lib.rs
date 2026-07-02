pub mod error;
pub mod tracing;

pub use error::AppError;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HelloWorldRequest {
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HelloWorldResponse {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
}