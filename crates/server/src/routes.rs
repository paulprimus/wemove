use axum::{
    extract::Extension,
    routing::get,
    Router,
};
use tower_http::trace::TraceLayer;
use std::net::SocketAddr;
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};

pub use crate::state::AppState;

/// Erstellt den Prometheus-Recorder/Exporter und installiert ihn als
/// globalen Metrics-Recorder. Gibt ein Handle zurück, mit dem der aktuelle
/// Metriken-Snapshot gerendert werden kann (z. B. für einen `/metrics`-Endpoint).
fn setup_metrics_recorder() -> PrometheusHandle {
    let recorder = PrometheusBuilder::new().build_recorder();
    let handle = recorder.handle();
    metrics::set_global_recorder(recorder).expect("failed to install Prometheus recorder");
    handle
}

/// Rendert den aktuellen Metriken-Snapshot als Prometheus-Textformat.
async fn metrics_handler(Extension(handle): Extension<PrometheusHandle>) -> String {
    handle.render()
}

pub async fn create_app(state: AppState) -> Router {
    let metrics_handle = setup_metrics_recorder();

    Router::new()
        .route("/", get(crate::handlers::hello_world).post(crate::handlers::hello_world_post))
        .route("/health", get(crate::handlers::health))
        .route("/metrics", get(metrics_handler))
        .layer(TraceLayer::new_for_http())
        .layer(Extension(metrics_handle))
        .with_state(state)
}
