mod auth_rest;
mod error;
mod handlers;
mod openapi;
mod routes;
mod state;
mod user_repo;

use common::tracing as common_tracing;
use config::{Args, AuthConfig};
use state::AppState;
use tokio::net::TcpListener;
use topcoat::serve;
use tracing;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::load();
    let (addr, log_level) = args.merge_with_config();
    let auth_config = AuthConfig::load();

    common_tracing::init_tracing(&log_level);

    tracing::info!("Starting server on {}", addr);

    let state = AppState::new(
        auth_config.jwt_secret.into_bytes(),
        auth_config.token_expiry_secs,
    )
    .await?;

    let api = routes::create_app(state.clone()).await;
    let app = web::register(topcoat::router::Router::builder().route(
        topcoat::router::tower::TowerRoute::new(
            topcoat::router::Methods::Any,
            topcoat::router::Path::new("/{*rest}"),
            api,
        ),
    ))
    .build();

    let listener = TcpListener::bind(addr).await?;
    serve(listener, app).await?;

    Ok(())
}