use axum::{extract::Extension, routing::get, Router};
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

pub async fn create_app(
    state: AppState,
    jwt_secret: impl AsRef<[u8]>,
    token_expiry_secs: u64,
) -> Router {
    let metrics_handle = setup_metrics_recorder();

    let jwt_secret_vec = jwt_secret.as_ref().to_vec();

    let auth_router = marvels_auth::AuthRouterBuilder::new()
        .jwt_secret(&jwt_secret_vec)
        .token_expiry(token_expiry_secs)
        .build();

    let auth_state = marvels_auth::AppState::new(jwt_secret_vec, token_expiry_secs);

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
        .route("/metrics", get(metrics_handler))
        .nest_service("/auth", auth_router.clone())
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(TraceLayer::new_for_http())
        .layer(Extension(metrics_handle))
        .layer(Extension(auth_router))
        .layer(Extension(auth_state))
        .with_state(state)
}
