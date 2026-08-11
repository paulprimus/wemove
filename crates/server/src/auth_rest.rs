use axum::{
    extract::{Extension, State},
    response::Response,
    Json,
};
use marvels_auth::rest::{JsonErrorResponse, JsonTokenRequest, JsonTokenResponse};

pub use marvels_auth::AppState;

#[utoipa::path(
    post,
    path = "/api/auth/token",
    tag = "auth",
    request_body = JsonTokenRequest,
    responses(
        (status = 200, description = "Access Token ausgestellt", body = JsonTokenResponse),
        (status = 400, description = "Ungültige Anfrage", body = JsonErrorResponse),
        (status = 401, description = "Authentifizierung fehlgeschlagen", body = JsonErrorResponse),
        (status = 500, description = "Serverfehler", body = JsonErrorResponse)
    )
)]
pub async fn token(
    Extension(auth_state): Extension<AppState>,
    Json(payload): Json<JsonTokenRequest>,
) -> Response {
    marvels_auth::rest::token(State(auth_state), Json(payload)).await
}
