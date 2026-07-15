mod auth_proxy;
mod error;
mod handlers;
mod openapi;
mod routes;
mod state;

use axum::serve;
use common::tracing as common_tracing;
use config::{Args, AuthConfig};
use tokio::net::TcpListener;
use tracing;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::load();
    let (addr, log_level) = args.merge_with_config();
    let auth_config = AuthConfig::load();

    common_tracing::init_tracing(&log_level);

    tracing::info!("Starting server on {}", addr);

    let state = routes::AppState::default();
    let app = routes::create_app(state, auth_config.jwt_secret.as_bytes(), auth_config.token_expiry_secs).await;

    let listener = TcpListener::bind(addr).await?;
    serve(listener, app).await?;

    Ok(())
}