use axum::{
    routing::{get, post},
    Router,
};
use tower_http::trace::TraceLayer;
use std::net::SocketAddr;
use axum_prometheus::PrometheusBuilder;

pub use crate::middleware::AppState;

pub fn create_app(state: AppState) -> Router {
    let (prometheus_layer, _) = PrometheusBuilder::new().build_pair();

    Router::new()
        .route("/", get(crate::handlers::hello_world).post(crate::handlers::hello_world_post))
        .route("/health", get(crate::handlers::health))
        .layer(TraceLayer::new_for_http())
        .layer(prometheus_layer)
        .with_state(state)
}

pub fn create_addr(host: String, port: u16) -> SocketAddr {
    format!("{}:{}", host, port).parse().unwrap()
}