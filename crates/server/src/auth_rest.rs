use axum::{
    extract::Extension,
    http::StatusCode,
    response::{IntoResponse, Response},
    Form, Json,
};
use common::{LoginRequest, LoginResponse, RegisterRequest, RegisterResponse};
use marvels_auth::rest::{JsonErrorResponse, JsonTokenRequest};
use serde::Serialize;
use utoipa::ToSchema;

use crate::state::AppState as ServerAppState;
use crate::user_repo::{CreateUser, UserRepository};
use marvels_auth::AppState as AuthAppState;

#[derive(Debug, Serialize, ToSchema)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub scope: String,
}

#[utoipa::path(
    post,
    path = "/api/auth/token",
    tag = "auth",
    request_body = JsonTokenRequest,
    responses(
        (status = 200, description = "Access Token ausgestellt", body = TokenResponse),
        (status = 400, description = "Ungültige Anfrage", body = JsonErrorResponse),
        (status = 401, description = "Authentifizierung fehlgeschlagen", body = JsonErrorResponse),
        (status = 500, description = "Serverfehler", body = JsonErrorResponse)
    )
)]
pub async fn token(
    Extension(auth_state): Extension<AuthAppState>,
    Json(payload): Json<JsonTokenRequest>,
) -> Response {
    let scope = payload.scope.unwrap_or_else(|| "read".to_string());

    let access_token = match auth_state.create_access_token(&payload.client_id, &scope) {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(JsonErrorResponse {
                    error: "server_error".to_string(),
                    error_description: e.to_string(),
                }),
            )
                .into_response();
        }
    };

    (
        StatusCode::OK,
        Json(TokenResponse {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in: auth_state.token_expiry_secs as i64,
            scope,
        }),
    )
        .into_response()
}

fn unauthorized_response(message: &str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(LoginResponse {
            success: false,
            message: message.to_string(),
            token: None,
        }),
    )
        .into_response()
}

fn internal_error_response(message: &str) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(LoginResponse {
            success: false,
            message: message.to_string(),
            token: None,
        }),
    )
        .into_response()
}

fn bad_request_response(message: &str) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(RegisterResponse {
            success: false,
            message: message.to_string(),
            user_id: None,
        }),
    )
        .into_response()
}

fn conflict_response(message: &str) -> Response {
    (
        StatusCode::CONFLICT,
        Json(RegisterResponse {
            success: false,
            message: message.to_string(),
            user_id: None,
        }),
    )
        .into_response()
}

fn register_success_response(user_id: i64) -> Response {
    (
        StatusCode::CREATED,
        Json(RegisterResponse {
            success: true,
            message: "Registration successful".to_string(),
            user_id: Some(user_id),
        }),
    )
        .into_response()
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
    Extension(auth_state): Extension<AuthAppState>,
    Extension(server_state): Extension<ServerAppState>,
    Form(payload): Form<LoginRequest>,
) -> Response {
    tracing::info!("Login attempt for email: {}", payload.email);

    if payload.email.is_empty() || payload.password.is_empty() {
        return unauthorized_response("Email and password are required");
    }

    let repo = UserRepository::new(server_state.db.clone());

    let user = match repo.find_by_email(&payload.email).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            tracing::info!("User not found: {}", payload.email);
            return unauthorized_response("Invalid email or password");
        }
        Err(e) => {
            tracing::error!("Failed to find user: {}", e);
            return internal_error_response("Database error");
        }
    };

    let password_valid = match repo.verify_password(&payload.password, &user.password_hash) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Password verification error: {}", e);
            return internal_error_response("Authentication error");
        }
    };

    if !password_valid {
        tracing::info!("Invalid password for email: {}", payload.email);
        return unauthorized_response("Invalid email or password");
    }

    let scope = "read write";
    let access_token = match auth_state.create_access_token(&payload.email, scope) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Failed to create access token: {}", e);
            return internal_error_response("Failed to create token");
        }
    };

    tracing::info!("Login successful for email: {}", payload.email);

    (
        StatusCode::OK,
        Json(LoginResponse {
            success: true,
            message: "Login successful".to_string(),
            token: Some(access_token),
        }),
    )
        .into_response()
}

#[utoipa::path(
    post,
    path = "/auth/register",
    tag = "auth",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "Registration successful", body = RegisterResponse),
        (status = 400, description = "Invalid request", body = RegisterResponse),
        (status = 409, description = "Email already exists", body = RegisterResponse)
    )
)]
pub async fn register(
    Extension(server_state): Extension<ServerAppState>,
    Form(payload): Form<RegisterRequest>,
) -> Response {
    tracing::info!("Registration attempt for email: {}", payload.email);

    if payload.email.is_empty() || payload.password.is_empty() || payload.name.is_empty() {
        return bad_request_response("Name, email and password are required");
    }

    let repo = UserRepository::new(server_state.db.clone());

    let create_user = CreateUser {
        email: payload.email,
        name: payload.name,
        password: payload.password,
    };

    match repo.create(create_user).await {
        Ok(user_id) => {
            tracing::info!("Registration successful for user_id: {}", user_id);
            register_success_response(user_id)
        }
        Err(common::error::AppError::Conflict(msg)) => {
            tracing::info!("Email already exists");
            conflict_response(&msg)
        }
        Err(e) => {
            tracing::error!("Registration error: {}", e);
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
}