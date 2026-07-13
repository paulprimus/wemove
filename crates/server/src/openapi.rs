use utoipa::OpenApi;
use common::{MainRequest, MainResponse, HealthResponse};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "WeMove API",
        description = "REST API for the WeMove application"
    ),
    paths(super::handlers::main_get, super::handlers::main_post, super::handlers::health),
    components(schemas(MainRequest, MainResponse, HealthResponse))
)]
pub struct ApiDoc;