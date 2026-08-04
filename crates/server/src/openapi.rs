use utoipa::OpenApi;
use common::{MainRequest, MainResponse, HealthResponse};
use marvels_auth::rest::{JsonTokenRequest, JsonTokenResponse, JsonErrorResponse};

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
    components(schemas(MainRequest, MainResponse, HealthResponse, JsonTokenRequest, JsonTokenResponse, JsonErrorResponse))
)]
pub struct ApiDoc;