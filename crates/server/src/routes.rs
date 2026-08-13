use axum::{
    extract::Extension,
    routing::get,
    Router,
};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub use crate::openapi::ApiDoc;
pub use crate::state::AppState;

fn setup_metrics_recorder() -> PrometheusHandle {
    let recorder = PrometheusBuilder::new().build_recorder();
    let handle = recorder.handle();
    metrics::set_global_recorder(recorder).expect("failed to install Prometheus recorder");
    handle
}

async fn metrics_handler(Extension(handle): Extension<PrometheusHandle>) -> String {
    handle.render()
}

pub async fn create_app(state: AppState) -> Router {
    let metrics_handle = setup_metrics_recorder();

    Router::new()
        .route(
            "/api/main",
            get(crate::handlers::main_get).post(crate::handlers::main_post),
        )
        .route("/api/health", get(crate::handlers::health))
        .route(
            "/api/auth/token",
            axum::routing::post(crate::auth_rest::token),
        )
        .route(
            "/api/auth/login",
            axum::routing::post(crate::auth_rest::login),
        )
        .route(
            "/api/auth/register",
            axum::routing::post(crate::auth_rest::register),
        )
        .route("/metrics", get(metrics_handler))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(TraceLayer::new_for_http())
        .layer(Extension(metrics_handle))
        .layer(Extension(state.clone()))
        .with_state(state)
}