use common::{HealthResponse, MainRequest, MainResponse};
use marvels_auth::rest::{JsonErrorResponse, JsonTokenRequest, JsonTokenResponse};
use utoipa::OpenApi;

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
        JsonTokenResponse,
        JsonErrorResponse
    ))
)]
pub struct ApiDoc;
