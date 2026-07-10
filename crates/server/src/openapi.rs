use utoipa::OpenApi;
use common::{HelloWorldRequest, HelloWorldResponse, HealthResponse};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "WeMove API",
        description = "REST API for the WeMove application"
    ),
    paths(super::handlers::hello_world, super::handlers::hello_world_post, super::handlers::health),
    components(schemas(HelloWorldRequest, HelloWorldResponse, HealthResponse))
)]
pub struct ApiDoc;