use utoipa::OpenApi;
use common::{MainRequest, MainResponse, HealthResponse};
use crate::auth_proxy::{TokenRequest, TokenResponse, TokenErrorResponse};

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
        super::auth_proxy::token
    ),
    components(schemas(MainRequest, MainResponse, HealthResponse, TokenRequest, TokenResponse, TokenErrorResponse))
)]
pub struct ApiDoc;