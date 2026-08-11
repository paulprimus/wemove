use common::{HealthResponse, MainRequest, MainResponse};
use marvels_auth::rest::{JsonErrorResponse, JsonTokenRequest};
use utoipa::OpenApi;

use super::auth_rest::TokenResponse;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "WeMove API",
        description = "REST API for the WeMove application"
    ),
    paths(
        super::handlers::main_get,
        super::handlers::main_post,
        super::handlers::health,
        super::auth_rest::token
    ),
    components(schemas(
        MainRequest,
        MainResponse,
        HealthResponse,
        JsonTokenRequest,
        TokenResponse,
        JsonErrorResponse
    ))
)]
pub struct ApiDoc;
