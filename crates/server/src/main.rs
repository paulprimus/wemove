mod auth_rest;
mod error;
mod handlers;
mod openapi;
mod routes;
mod state;
mod user_repo;

use common::tracing as common_tracing;
use common::{LoginRequest, RegisterRequest};
use config::{Args, AuthConfig};
use state::AppState;
use tokio::net::TcpListener;
use topcoat::context::Cx;
use topcoat::cookie::RouterBuilderCookieExt;
use topcoat::router::content::Form;
use topcoat::router::error::{redirect, see_other, SeeOther};
use topcoat::router::{Methods, Path, RouterBuilder, route};
use topcoat::router::tower::TowerRoute;
use topcoat::session::{self, Session};
use topcoat::session::RouterBuilderSessionExt;
use topcoat::serve;
use tracing;
use topcoat::Result;

use user_repo::{CreateUser, UserRepository};
use web::app::app_layout;
use web::auth::login::login as login_page;
use web::auth::register::register as register_page;
use web::dashboard::dashboard as dashboard_page;
use web::home::index;

fn hash_to_hex(hash: &[u8]) -> String {
    hash.iter().map(|b| format!("{:02x}", b)).collect()
}

#[route(POST "/auth/login")]
async fn login(cx: &Cx, Form(payload): Form<LoginRequest>) -> Result<SeeOther> {
    if payload.email.is_empty() || payload.password.is_empty() {
        return Err(redirect("/login?error=missing_fields").into());
    }

    let state: &AppState = match topcoat::context::app_context(cx) {
        Some(s) => s,
        None => return Err(redirect("/login?error=internal_error").into()),
    };

    let repo = UserRepository::new(state.db.clone());

    let user = match repo.find_by_email(&payload.email).await {
        Ok(Some(u)) => u,
        _ => return Err(redirect("/login?error=invalid_credentials").into()),
    };

    let password_valid = match repo.verify_password(&payload.password, &user.password_hash) {
        Ok(v) => v,
        _ => return Err(redirect("/login?error=invalid_credentials").into()),
    };

    if !password_valid {
        return Err(redirect("/login?error=invalid_credentials").into());
    }

    let session = match session::start(cx).await {
        Ok(s) => s,
        Err(_) => return Err(redirect("/login?error=session_error").into()),
    };

    if let Err(e) = persist_session(state, user.id, &session).await {
        tracing::error!("Failed to persist session: {}", e);
    }

    Ok(see_other("/dashboard"))
}

async fn persist_session(state: &AppState, user_id: i64, session: &Session) -> Result<(), String> {
    let conn = state.db.connect().map_err(|e| e.to_string())?;

    let token_hash = hash_to_hex(session.token_hash.as_ref());
    let expires_at = session.expires_at
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs() as i64;

    let _ = conn.execute(
        "INSERT INTO sessions (token_hash, user_id, expires_at) VALUES (?1, ?2, ?3)",
        (token_hash, user_id, expires_at),
    ).await;

    Ok(())
}

#[route(POST "/auth/register")]
async fn register(cx: &Cx, Form(payload): Form<RegisterRequest>) -> Result<SeeOther> {
    if payload.email.is_empty() || payload.password.is_empty() || payload.name.is_empty() {
        return Err(redirect("/register?error=missing_fields").into());
    }

    let state: &AppState = match topcoat::context::app_context(cx) {
        Some(s) => s,
        None => return Err(redirect("/register?error=internal_error").into()),
    };

    let repo = UserRepository::new(state.db.clone());

    let create_user = CreateUser {
        email: payload.email,
        name: payload.name,
        password: payload.password,
    };

    match repo.create(create_user).await {
        Ok(_) => Ok(see_other("/login?registered=true")),
        Err(common::error::AppError::Conflict(_)) => {
            Err(redirect("/register?error=email_exists").into())
        }
        Err(e) => {
            tracing::error!("Registration error: {}", e);
            Err(redirect("/register?error=internal_error").into())
        }
    }
}

#[route(POST "/auth/logout")]
async fn logout(cx: &Cx) -> Result<SeeOther> {
    if let Some(hash) = session::stop(cx).await.ok().flatten() {
        let state = topcoat::context::app_context::<AppState>(cx);
        if let Ok(conn) = state.db.connect() {
            let token_hash = hash_to_hex(hash.as_ref());
            let _ = conn.execute(
                "DELETE FROM sessions WHERE token_hash = ?1",
                [token_hash],
            ).await;
        }
    }
    Ok(see_other("/"))
}

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

    let app = RouterBuilder::new()
        .layout(app_layout)
        .page(index)
        .page(login_page)
        .page(register_page)
        .page(dashboard_page)
        .cookies()
        .app_context(state)
        .sessions(topcoat::session::SessionConfig::default())
        .route(login)
        .route(register)
        .route(logout)
        .route(TowerRoute::new(Methods::Any, Path::new("/{*rest}"), api))
        .build();

    let listener = TcpListener::bind(addr).await?;
    serve(listener, app).await?;

    Ok(())
}