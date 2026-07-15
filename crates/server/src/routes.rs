use axum::{
    extract::Extension,
    routing::get,
    Router,
};
use tower_http::trace::TraceLayer;
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use utoipa::{OpenApi};
use utoipa_swagger_ui::SwaggerUi;

pub use crate::state::AppState;
pub use crate::openapi::ApiDoc;

fn setup_metrics_recorder() -> PrometheusHandle {
    let recorder = PrometheusBuilder::new().build_recorder();
    let handle = recorder.handle();
    metrics::set_global_recorder(recorder).expect("failed to install Prometheus recorder");
    handle
}

async fn metrics_handler(Extension(handle): Extension<PrometheusHandle>) -> String {
    handle.render()
}

pub async fn create_app(state: AppState, jwt_secret: impl AsRef<[u8]>, token_expiry_secs: u64) -> Router {
    let metrics_handle = setup_metrics_recorder();

    let auth_router = marvels_auth::AuthRouterBuilder::new()
        .jwt_secret(jwt_secret)
        .token_expiry(token_expiry_secs)
        .build();

    Router::new()
        .route("/api/main", get(crate::handlers::main_get).post(crate::handlers::main_post))
        .route("/api/health", get(crate::handlers::health))
        .route("/api/auth/token", axum::routing::post(crate::auth_proxy::token))
        .route("/metrics", get(metrics_handler))
        .nest_service("/auth", auth_router.clone())
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(TraceLayer::new_for_http())
        .layer(Extension(metrics_handle))
        .layer(Extension(auth_router))
        .with_state(state)
}