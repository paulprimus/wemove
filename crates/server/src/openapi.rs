use common::{
    HealthResponse, LoginRequest, LoginResponse, MainRequest, MainResponse, RegisterRequest,
    RegisterResponse,
};
use utoipa::OpenApi;

use super::auth_rest::{JsonErrorResponse, JsonTokenRequest, TokenResponse};

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
        super::auth_rest::login,
        super::auth_rest::register
    ),
    components(schemas(
        MainRequest,
        MainResponse,
        HealthResponse,
        JsonTokenRequest,
        TokenResponse,
        JsonErrorResponse,
        LoginRequest,
        LoginResponse,
        RegisterRequest,
        RegisterResponse
    ))
)]
pub struct ApiDoc;
