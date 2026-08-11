use axum::{
    extract::Extension,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use marvels_auth::rest::{JsonErrorResponse, JsonTokenRequest};
use serde::Serialize;
use utoipa::ToSchema;

pub use marvels_auth::AppState;

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
    Extension(auth_state): Extension<AppState>,
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
