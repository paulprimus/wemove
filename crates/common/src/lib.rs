pub mod error;
pub mod tracing;

pub use error::AppError;
pub use error::DbError;

pub use serde::{Deserialize, Serialize};
pub use utoipa::ToSchema;

#[derive(Clone)]
pub struct AuthState {
    pub db: turso::Database,
}

#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MainRequest {
    #[schema(example = json!({"name": "Paul"}))]
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MainResponse {
    #[schema(example = json!("Hello, Paul!"))]
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    #[schema(example = json!("healthy"))]
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginRequest {
    #[schema(example = json!("user@example.com"))]
    pub email: String,
    #[schema(example = json!("password123"))]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginResponse {
    pub success: bool,
    #[schema(example = json!("Login successful"))]
    pub message: String,
    pub token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RegisterRequest {
    #[schema(example = json!("John Doe"))]
    pub name: String,
    #[schema(example = json!("user@example.com"))]
    pub email: String,
    #[schema(example = json!("password123"))]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RegisterResponse {
    pub success: bool,
    #[schema(example = json!("Registration successful"))]
    pub message: String,
    pub user_id: Option<i64>,
}
