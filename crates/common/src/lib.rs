pub mod error;
pub mod tracing;

pub use error::AppError;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HelloWorldRequest {
    #[schema(example = json!({"name": "Alice"}))]
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HelloWorldResponse {
    #[schema(example = json!("Hello, Alice!"))]
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    #[schema(example = json!("healthy"))]
    pub status: String,
}