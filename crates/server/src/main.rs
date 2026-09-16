mod auth_rest;
mod error;
mod handlers;
mod openapi;
mod routes;
mod state;
mod user_repo;

use std::time::Duration;

use common::tracing as common_tracing;
use common::{AuthState, LoginRequest, RegisterRequest};
use config::{Args, AuthConfig};
use state::AppState;
use tokio::net::TcpListener;
use topcoat::Result;
use topcoat::asset::{AssetBundle, RouterBuilderAssetExt};
use topcoat::context::Cx;
use topcoat::cookie::RouterBuilderCookieExt;
use topcoat::router::content::Form;
use topcoat::router::error::{SeeOther, see_other};
use topcoat::router::tower::TowerRoute;
use topcoat::router::{Methods, Path, RouterBuilder, page, route};
use topcoat::serve;
use topcoat::session::RouterBuilderSessionExt;
use topcoat::session::{self, Session};
use topcoat::view::{View, view};
use tracing;

use user_repo::{CreateUser, UserRepository};
use web::app::app_layout;
use web::auth::login::login as login_page;
use web::auth::register::register as register_page;
use web::home::index;

fn hash_to_hex(hash: &[u8]) -> String {
    hash.iter().map(|b| format!("{:02x}", b)).collect()
}

#[route(POST "/auth/login")]
async fn login(cx: &Cx, Form(payload): Form<LoginRequest>) -> Result<SeeOther> {
    if payload.email.is_empty() || payload.password.is_empty() {
        return Ok(see_other("/login?error=missing_fields"));
    }

    let state = topcoat::context::app_context::<AppState>(cx);

    let repo = UserRepository::new(state.db.clone());

    let user = match repo.find_by_email(&payload.email).await {
        Ok(Some(u)) => u,
        _ => return Ok(see_other("/login?error=invalid_credentials")),
    };

    let password_valid = match repo.verify_password(&payload.password, &user.password_hash) {
        Ok(v) => v,
        _ => return Ok(see_other("/login?error=invalid_credentials")),
    };

    if !password_valid {
        return Ok(see_other("/login?error=invalid_credentials"));
    }

    let session = match session::start(cx).await {
        Ok(s) => s,
        Err(_) => return Ok(see_other("/login?error=session_error")),
    };

    if let Err(e) = persist_session(state, user.id, &session).await {
        tracing::error!("Failed to persist session: {}", e);
        let _ = session::stop(cx).await;
        return Ok(see_other("/login?error=session_error"));
    }

    Ok(see_other("/dashboard"))
}

#[page("/dashboard")]
async fn dashboard(cx: &Cx) -> Result<impl View> {
    let state = topcoat::context::app_context::<AppState>(cx);
    let authenticated = match session::token_hash(cx).await? {
        Some(hash) => has_valid_session(state, &hash).await,
        None => false,
    };

    if !authenticated {
        return Err(topcoat::router::error::redirect("/login").into());
    }

    Ok(view! { web::dashboard::dashboard_view() })
}

async fn has_valid_session(state: &AppState, hash: &session::TokenHash) -> bool {
    let conn = match state.db.connect() {
        Ok(conn) => conn,
        Err(error) => {
            tracing::error!("Failed to connect to session database: {}", error);
            return false;
        }
    };

    let mut statement = match conn
        .prepare("SELECT expires_at FROM sessions WHERE token_hash = ?1")
        .await
    {
        Ok(statement) => statement,
        Err(error) => {
            tracing::error!("Failed to prepare session lookup: {}", error);
            return false;
        }
    };

    let mut rows = match statement.query([hash_to_hex(hash.as_ref())]).await {
        Ok(rows) => rows,
        Err(error) => {
            tracing::error!("Failed to query session: {}", error);
            return false;
        }
    };

    let expires_at = match rows.next().await {
        Ok(Some(row)) => match row.get_value(0) {
            Ok(turso::Value::Integer(expires_at)) => expires_at,
            Ok(_) => return false,
            Err(error) => {
                tracing::error!("Failed to read session expiry: {}", error);
                return false;
            }
        },
        Ok(None) => return false,
        Err(error) => {
            tracing::error!("Failed to read session result: {}", error);
            return false;
        }
    };

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs() as i64);

    expires_at > now
}

async fn persist_session(state: &AppState, user_id: i64, session: &Session) -> Result<(), String> {
    let conn = state.db.connect().map_err(|e| e.to_string())?;

    let token_hash = hash_to_hex(session.token_hash.as_ref());
    let expires_at = session
        .expires_at
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs() as i64;

    conn.execute(
        "INSERT INTO sessions (token_hash, user_id, expires_at) VALUES (?1, ?2, ?3)",
        (token_hash, user_id, expires_at),
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[route(POST "/auth/register")]
async fn register(cx: &Cx, Form(payload): Form<RegisterRequest>) -> Result<SeeOther> {
    if payload.email.is_empty() || payload.password.is_empty() || payload.name.is_empty() {
        return Ok(see_other("/register?error=missing_fields"));
    }

    let state = topcoat::context::app_context::<AppState>(cx);

    let repo = UserRepository::new(state.db.clone());

    let create_user = CreateUser {
        email: payload.email,
        name: payload.name,
        password: payload.password,
    };

    match repo.create(create_user).await {
        Ok(_) => Ok(see_other("/login?registered=true")),
        Err(common::error::AppError::Conflict(_)) => Ok(see_other("/register?error=email_exists")),
        Err(e) => {
            tracing::error!("Registration error: {}", e);
            Ok(see_other("/register?error=internal_error"))
        }
    }
}

#[route(POST "/auth/logout")]
async fn logout(cx: &Cx) -> Result<SeeOther> {
    if let Some(hash) = session::stop(cx).await? {
        let state = topcoat::context::app_context::<AppState>(cx);
        if let Ok(conn) = state.db.connect() {
            let token_hash = hash_to_hex(hash.as_ref());
            let _ = conn
                .execute("DELETE FROM sessions WHERE token_hash = ?1", [token_hash])
                .await;
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
    let auth_state = AuthState {
        db: state.db.clone(),
    };

    let app = RouterBuilder::new()
        .layout(app_layout)
        .page(index)
        .page(login_page)
        .page(register_page)
        .page(dashboard)
        .cookies()
        .app_context(auth_state)
        .app_context(state)
        .sessions(
            session::SessionConfig::builder()
                .lifetime(Duration::from_secs(24 * 60 * 60))
                .build(),
        )
        .route(login)
        .route(register)
        .route(logout)
        .route(TowerRoute::new(Methods::Any, Path::new("/{*rest}"), api))
        .assets(AssetBundle::load().expect("failed to load Topcoat asset bundle"))
        .build();

    let listener = TcpListener::bind(addr).await?;
    serve(listener, app).await?;

    Ok(())
}
