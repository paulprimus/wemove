//! Browser-freundlicher JSON-Proxy für den Marvels `client_credentials`-Grant.
//!
//! Marvels selbst spricht ausschließlich Protobuf (`/auth/authorize`), was aus
//! dem Browser heraus (fetch/JSON) unpraktisch ist. Dieser Handler nimmt ein
//! einfaches JSON-Body entgegen, baut daraus eine `AuthorizeRequest`-Protobuf-
//! Message und reicht sie **in-process** (kein echter Netzwerk-Hop) über
//! `tower::ServiceExt::oneshot` an den bereits gemounteten `marvels_auth`-
//! Router weiter. Die Protobuf-Antwort wird dekodiert und als JSON
//! zurückgegeben.

use axum::body::{to_bytes, Body};
use axum::extract::Extension;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{Json, Router};
use proto::authentication::security::{AuthorizeRequest, AuthorizeResponse};
use serde::{Deserialize, Serialize};
use tower::ServiceExt;
use utoipa::ToSchema;

/// Maximale Größe der Antwort, die von `marvels_auth` gelesen wird.
const MAX_RESPONSE_BYTES: usize = 64 * 1024;

#[derive(Debug, Deserialize, ToSchema)]
pub struct TokenRequest {
    pub client_id: String,
    pub client_secret: String,
    /// Leerzeichen-getrennte Scopes, z.B. "read write". Optional, Default "read".
    #[serde(default)]
    pub scope: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i32,
    pub scope: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TokenErrorResponse {
    pub error: String,
    pub error_description: String,
}

/// `POST /api/auth/token` — JSON-Wrapper um den `client_credentials`-Grant von Marvels.
#[utoipa::path(
    post,
    path = "/api/auth/token",
    tag = "auth",
    request_body = TokenRequest,
    responses(
        (status = 200, description = "Access Token ausgestellt", body = TokenResponse),
        (status = 400, description = "Ungültige Anfrage", body = TokenErrorResponse),
        (status = 401, description = "Client-Authentifizierung fehlgeschlagen", body = TokenErrorResponse),
        (status = 502, description = "Auth-Backend nicht erreichbar/ungültige Antwort", body = TokenErrorResponse)
    )
)]
pub async fn token(
    Extension(auth_router): Extension<Router>,
    Json(payload): Json<TokenRequest>,
) -> Response {
    let scope = payload.scope.unwrap_or_else(|| "read".to_string());

    let proto_request = AuthorizeRequest {
        grant_type: "client_credentials".to_string(),
        client_id: payload.client_id,
        scope,
        refresh_token: String::new(),
        code_verifier: String::new(),
        code: String::new(),
        redirect_uri: String::new(),
    };

    // Anmerkung: client_secret wird von Marvels' client_credentials-Handler aktuell
    // nicht validiert (siehe server.rs::authorize). Der Body wird dennoch mitgesendet,
    // sobald Marvels eine echte Client-Prüfung implementiert.
    let body_bytes = proto_request.encode_payload();

    let request = match Request::builder()
        .method("POST")
        .uri("/authorize")
        .header("content-type", "application/protobuf")
        .body(Body::from(body_bytes))
    {
        Ok(req) => req,
        Err(err) => {
            tracing::error!("Failed to build internal auth request: {err}");
            return internal_error_response();
        }
    };

    let response = match auth_router.oneshot(request).await {
        Ok(resp) => resp,
        Err(err) => {
            tracing::error!("Auth backend call failed: {err}");
            return internal_error_response();
        }
    };

    let status = response.status();
    let body_bytes = match to_bytes(response.into_body(), MAX_RESPONSE_BYTES).await {
        Ok(bytes) => bytes,
        Err(err) => {
            tracing::error!("Failed to read auth backend response: {err}");
            return internal_error_response();
        }
    };

    let decoded = match AuthorizeResponse::decode_payload(&body_bytes) {
        Ok(decoded) => decoded,
        Err(err) => {
            tracing::error!("Failed to decode auth backend response: {err}");
            return internal_error_response();
        }
    };

    if !decoded.error.is_empty() {
        let http_status = if status.is_success() {
            StatusCode::BAD_REQUEST
        } else {
            status
        };
        return (
            http_status,
            Json(TokenErrorResponse {
                error: decoded.error,
                error_description: decoded.error_description,
            }),
        )
            .into_response();
    }

    (
        StatusCode::OK,
        Json(TokenResponse {
            access_token: decoded.access_token,
            token_type: decoded.token_type,
            expires_in: decoded.expires_in,
            scope: decoded.scope,
        }),
    )
        .into_response()
}

fn internal_error_response() -> Response {
    (
        StatusCode::BAD_GATEWAY,
        Json(TokenErrorResponse {
            error: "server_error".to_string(),
            error_description: "Auth-Backend nicht erreichbar oder ungültige Antwort".to_string(),
        }),
    )
        .into_response()
}
