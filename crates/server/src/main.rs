mod handlers;
mod middleware;
mod routes;

use std::net::SocketAddr;
use axum::serve;
use tokio::net::TcpListener;
use common::tracing as common_tracing;
use config::Args;
use tracing;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::load();
    let (addr, log_level) = args.merge_with_config();

    common_tracing::init_tracing(&log_level);

    tracing::info!("Starting server on {}", addr);

    let state = routes::AppState::default();
    let app = routes::create_app(state).await;

    let listener = TcpListener::bind(addr).await?;
    serve(listener, app).await?;

    Ok(())
}