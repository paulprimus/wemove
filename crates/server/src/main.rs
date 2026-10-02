mod auth_rest;
mod board_repo;
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
use topcoat::router::error::internal_server_error;
use topcoat::router::error::{SeeOther, see_other};
use topcoat::router::tower::TowerRoute;
use topcoat::router::{Methods, Path, RouterBuilder, page, parse_query_params, route};
use topcoat::runtime::RouterBuilderRuntimeExt;
use topcoat::serve;
use topcoat::session::RouterBuilderSessionExt;
use topcoat::session::{self, Session};
use topcoat::view::{View, view};
use tracing;

use user_repo::{CreateUser, UserRepository};
use web::app::app_layout;
use web::auth::login::login as login_page;
use web::auth::register::register as register_page;
use web::boards::new::create_board_form;
use web::home::index;

#[derive(serde::Deserialize)]
struct BoardPageQuery {
    error: Option<String>,
}

#[derive(serde::Deserialize)]
struct CreateBoardForm {
    name: String,
}

#[derive(Debug, PartialEq)]
enum BoardNameError {
    Empty,
    TooLong,
}

fn validate_board_name(name: &str) -> std::result::Result<&str, BoardNameError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(BoardNameError::Empty);
    }
    if name.chars().count() > 100 {
        return Err(BoardNameError::TooLong);
    }
    Ok(name)
}

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
    let Some(hash) = session::token_hash(cx).await? else {
        return Err(topcoat::router::error::redirect("/login").into());
    };
    let user_id = match session_user_id(state, &hash).await {
        Ok(Some(user_id)) => user_id,
        Ok(None) => return Err(topcoat::router::error::redirect("/login").into()),
        Err(error) => {
            tracing::error!("Failed to verify dashboard session: {}", error);
            return Err(internal_server_error(std::io::Error::other(error)).into());
        }
    };
    let boards = board_repo::BoardRepository::new(state.db.clone())
        .list_for_user(user_id)
        .await
        .map_err(|error| {
            tracing::error!("Failed to load user boards: {}", error);
            internal_server_error(error)
        })?;

    Ok(view! { web::dashboard::dashboard_view(boards: boards) })
}

#[page("/boards/new")]
async fn new_board_page(cx: &Cx) -> Result<impl View> {
    let state = topcoat::context::app_context::<AppState>(cx);
    let Some(hash) = session::token_hash(cx).await? else {
        return Err(topcoat::router::error::redirect("/login").into());
    };
    match session_user_id(state, &hash).await {
        Ok(Some(_)) => {}
        Ok(None) => return Err(topcoat::router::error::redirect("/login").into()),
        Err(error) => {
            tracing::error!("Failed to verify board creation session: {}", error);
            return Err(internal_server_error(std::io::Error::other(error)).into());
        }
    }

    let error = parse_query_params::<BoardPageQuery>(cx)
        .ok()
        .and_then(|query| query.error);

    Ok(view! { create_board_form(error: error) })
}

async fn session_user_id(
    state: &AppState,
    hash: &session::TokenHash,
) -> Result<Option<i64>, String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_secs() as i64;
    let conn = state.db.connect().map_err(|error| error.to_string())?;
    let mut statement = conn
        .prepare("SELECT user_id FROM sessions WHERE token_hash = ?1 AND expires_at > ?2")
        .await
        .map_err(|error| error.to_string())?;
    let mut rows = statement
        .query((hash_to_hex(hash.as_ref()), now))
        .await
        .map_err(|error| error.to_string())?;

    match rows.next().await.map_err(|error| error.to_string())? {
        Some(row) => match row.get_value(0).map_err(|error| error.to_string())? {
            turso::Value::Integer(user_id) => Ok(Some(user_id)),
            value => Err(format!("Invalid session user ID value: {value:?}")),
        },
        None => Ok(None),
    }
}

#[route(POST "/boards/new")]
async fn create_board(cx: &Cx, Form(payload): Form<CreateBoardForm>) -> Result<SeeOther> {
    let name = match validate_board_name(&payload.name) {
        Ok(name) => name,
        Err(BoardNameError::Empty) => return Ok(see_other("/boards/new?error=empty_name")),
        Err(BoardNameError::TooLong) => return Ok(see_other("/boards/new?error=name_too_long")),
    };

    let state = topcoat::context::app_context::<AppState>(cx);
    let Some(hash) = session::token_hash(cx).await? else {
        return Ok(see_other("/login"));
    };
    let user_id = match session_user_id(state, &hash).await {
        Ok(Some(user_id)) => user_id,
        Ok(None) => return Ok(see_other("/login")),
        Err(error) => {
            tracing::error!("Failed to verify board creation session: {}", error);
            return Err(internal_server_error(std::io::Error::other(error)).into());
        }
    };

    if let Err(error) = board_repo::BoardRepository::new(state.db.clone())
        .create_for_user(user_id, name)
        .await
    {
        tracing::error!("Failed to create board for user {}: {}", user_id, error);
        return Ok(see_other("/boards/new?error=storage"));
    }

    Ok(see_other("/dashboard"))
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
        .page(new_board_page)
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
        .route(create_board)
        .route(logout)
        .route(TowerRoute::new(Methods::Any, Path::new("/{*rest}"), api))
        .assets(AssetBundle::load().expect("failed to load Topcoat asset bundle"))
        .runtime()
        .build();

    let listener = TcpListener::bind(addr).await?;
    serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn board_names_are_trimmed_and_must_not_be_empty() {
        assert_eq!(validate_board_name("  Roadmap  "), Ok("Roadmap"));
        assert_eq!(validate_board_name(" \t "), Err(BoardNameError::Empty));
    }

    #[test]
    fn board_names_are_limited_to_one_hundred_characters() {
        assert!(validate_board_name(&"a".repeat(100)).is_ok());
        assert_eq!(
            validate_board_name(&"a".repeat(101)),
            Err(BoardNameError::TooLong)
        );
    }
}
