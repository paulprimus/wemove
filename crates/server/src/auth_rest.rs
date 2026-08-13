use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Form, Json,
};
use common::{LoginRequest, LoginResponse, RegisterRequest, RegisterResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::state::AppState;
use crate::user_repo::{CreateUser, UserRepository};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct JsonTokenRequest {
    pub client_id: String,
    #[serde(default)]
    pub client_secret: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct JsonErrorResponse {
    pub error: String,
    pub error_description: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub scope: String,
}

impl IntoResponse for TokenResponse {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

#[derive(Debug)]
pub struct TokenOk(pub TokenResponse);

impl IntoResponse for TokenOk {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, Json(self.0)).into_response()
    }
}

impl IntoResponse for JsonErrorResponse {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(self)).into_response()
    }
}

#[derive(Debug)]
pub struct TokenError;

impl IntoResponse for TokenError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("Token creation failed");
        (StatusCode::INTERNAL_SERVER_ERROR, Json(JsonErrorResponse {
            error: "server_error".to_string(),
            error_description: "Token creation failed".to_string(),
        })).into_response()
    }
}

#[utoipa::path(
    post,
    path = "/api/auth/token",
    tag = "auth",
    request_body = JsonTokenRequest,
    responses(
        (status = 200, description = "Access Token ausgestellt", body = TokenResponse),
        (status = 500, description = "Serverfehler", body = JsonErrorResponse)
    )
)]
pub async fn token(
    State(state): State<AppState>,
    Form(payload): Form<JsonTokenRequest>,
) -> Result<TokenOk, TokenError> {
    let scope = payload.scope.unwrap_or_else(|| "read".to_string());
    let access_token = state
        .create_access_token(&payload.client_id, &scope)
        .map_err(|_| TokenError)?;
    Ok(TokenOk(TokenResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: state.token_expiry_secs as i64,
        scope,
    }))
}

#[derive(Debug)]
pub struct LoginUnauthorized(pub String);

impl IntoResponse for LoginUnauthorized {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::UNAUTHORIZED,
            Json(LoginResponse {
                success: false,
                message: self.0,
                token: None,
            }),
        )
            .into_response()
    }
}

#[derive(Debug)]
pub struct LoginError;

impl IntoResponse for LoginError {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(LoginResponse {
                success: false,
                message: "Internal server error".to_string(),
                token: None,
            }),
        )
            .into_response()
    }
}

#[derive(Debug)]
pub struct LoginSuccess(pub String);

impl IntoResponse for LoginSuccess {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::OK,
            Json(LoginResponse {
                success: true,
                message: "Login successful".to_string(),
                token: Some(self.0),
            }),
        )
            .into_response()
    }
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials", body = LoginResponse)
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Form(payload): Form<LoginRequest>,
) -> Result<LoginSuccess, LoginUnauthorized> {
    tracing::info!("Login attempt for email: {}", payload.email);

    if payload.email.is_empty() || payload.password.is_empty() {
        return Err(LoginUnauthorized("Email and password are required".to_string()));
    }

    let repo = UserRepository::new(state.db.clone());

    let user = match repo.find_by_email(&payload.email).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            tracing::info!("User not found: {}", payload.email);
            return Err(LoginUnauthorized("Invalid email or password".to_string()));
        }
        Err(e) => {
            tracing::error!("Failed to find user: {}", e);
            return Err(LoginUnauthorized("Invalid email or password".to_string()));
        }
    };

    let password_valid = match repo.verify_password(&payload.password, &user.password_hash) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Password verification error: {}", e);
            return Err(LoginUnauthorized("Authentication error".to_string()));
        }
    };

    if !password_valid {
        tracing::info!("Invalid password for email: {}", payload.email);
        return Err(LoginUnauthorized("Invalid email or password".to_string()));
    }

    let scope = "read write";
    match state.create_access_token(&payload.email, scope) {
        Ok(access_token) => {
            tracing::info!("Login successful for email: {}", payload.email);
            Ok(LoginSuccess(access_token))
        }
        Err(e) => {
            tracing::error!("Failed to create access token: {}", e);
            Err(LoginUnauthorized("Authentication error".to_string()))
        }
    }
}

#[derive(Debug)]
pub struct RegisterBadRequest(pub String);

impl IntoResponse for RegisterBadRequest {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::BAD_REQUEST,
            Json(RegisterResponse {
                success: false,
                message: self.0,
                user_id: None,
            }),
        )
            .into_response()
    }
}

#[derive(Debug)]
pub struct RegisterConflict(pub String);

impl IntoResponse for RegisterConflict {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::CONFLICT,
            Json(RegisterResponse {
                success: false,
                message: self.0,
                user_id: None,
            }),
        )
            .into_response()
    }
}

#[derive(Debug)]
pub struct RegisterSuccess(pub i64);

impl IntoResponse for RegisterSuccess {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::CREATED,
            Json(RegisterResponse {
                success: true,
                message: "Registration successful".to_string(),
                user_id: Some(self.0),
            }),
        )
            .into_response()
    }
}

#[derive(Debug)]
pub struct RegisterError;

impl IntoResponse for RegisterError {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(RegisterResponse {
                success: false,
                message: "Failed to create user".to_string(),
                user_id: None,
            }),
        )
            .into_response()
    }
}

#[utoipa::path(
    post,
    path = "/api/auth/register",
    tag = "auth",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "Registration successful", body = RegisterResponse),
        (status = 400, description = "Invalid request", body = RegisterResponse),
        (status = 409, description = "Email already exists", body = RegisterResponse)
    )
)]
pub async fn register(
    State(state): State<AppState>,
    Form(payload): Form<RegisterRequest>,
) -> Result<RegisterSuccess, RegisterError> {
    tracing::info!("Registration attempt for email: {}", payload.email);

    if payload.email.is_empty() || payload.password.is_empty() || payload.name.is_empty() {
        return Err(RegisterError);
    }

    let repo = UserRepository::new(state.db.clone());

    let create_user = CreateUser {
        email: payload.email,
        name: payload.name,
        password: payload.password,
    };

    match repo.create(create_user).await {
        Ok(user_id) => {
            tracing::info!("Registration successful for user_id: {}", user_id);
            Ok(RegisterSuccess(user_id))
        }
        Err(common::error::AppError::Conflict(msg)) => {
            tracing::info!("Email already exists: {}", msg);
            Err(RegisterError)
        }
        Err(e) => {
            tracing::error!("Registration error: {}", e);
            Err(RegisterError)
        }
    }
}