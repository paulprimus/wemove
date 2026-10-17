mod auth_rest;
mod board_repo;
mod error;
mod handlers;
mod openapi;
mod project_repo;
mod routes;
mod state;
mod user_repo;
mod workspace_repo;

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
use topcoat::router::{Methods, Path, RouterBuilder, page, parse_query_params, path_param, route};
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
use web::projects::new::create_project_form;
use web::workspaces::new::create_workspace_form;

#[derive(serde::Deserialize)]
struct BoardPageQuery {
    error: Option<String>,
}

#[derive(serde::Deserialize)]
struct CreateBoardForm {
    name: String,
}

#[derive(serde::Deserialize)]
struct CreateProjectForm {
    name: String,
}

#[derive(serde::Deserialize)]
struct CreateWorkspaceForm {
    name: String,
}

path_param!(board_id: i64, error = not_found);
path_param!(project_id: i64, error = not_found);
path_param!(workspace_id: i64, error = not_found);

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
    let workspaces = workspace_repo::WorkspaceRepository::new(state.db.clone())
        .list_for_user(user_id)
        .await
        .map_err(|error| {
            tracing::error!("Failed to load user workspaces: {}", error);
            internal_server_error(error)
        })?;

    let workspaces = workspaces
        .into_iter()
        .map(|workspace| web::dashboard::WorkspaceSummary {
            id: workspace.id,
            name: workspace.name,
        })
        .collect();

    Ok(view! { web::dashboard::dashboard_view(workspaces: workspaces) })
}

#[page("/workspaces/new")]
async fn new_workspace_page(cx: &Cx) -> Result<impl View> {
    let state = topcoat::context::app_context::<AppState>(cx);
    let Some(hash) = session::token_hash(cx).await? else {
        return Err(topcoat::router::error::redirect("/login").into());
    };
    if session_user_id(state, &hash)
        .await
        .map_err(|error| internal_server_error(std::io::Error::other(error)))?
        .is_none()
    {
        return Err(topcoat::router::error::redirect("/login").into());
    }
    let error = parse_query_params::<BoardPageQuery>(cx)
        .ok()
        .and_then(|query| query.error);
    Ok(view! { create_workspace_form(error: error) })
}

#[page("/workspaces/{workspace_id}")]
async fn workspace_page(cx: &Cx) -> Result<impl View> {
    let state = topcoat::context::app_context::<AppState>(cx);
    let Some(hash) = session::token_hash(cx).await? else {
        return Err(topcoat::router::error::redirect("/login").into());
    };
    let user_id = session_user_id(state, &hash)
        .await
        .map_err(|error| internal_server_error(std::io::Error::other(error)))?
        .ok_or_else(|| topcoat::router::error::redirect("/login"))?;
    let id = *path_param::<WorkspaceId>(cx)?;
    let workspace = workspace_repo::WorkspaceRepository::new(state.db.clone())
        .find_for_user(user_id, id)
        .await
        .map_err(internal_server_error)?
        .ok_or_else(topcoat::router::error::not_found)?;
    let projects = project_repo::ProjectRepository::new(state.db.clone())
        .list_for_workspace(user_id, id)
        .await
        .map_err(internal_server_error)?
        .into_iter()
        .map(|project| web::workspaces::view::ProjectSummary {
            id: project.id,
            name: project.name,
        })
        .collect();
    Ok(
        view! { web::workspaces::view::workspace_view(id: workspace.id, name: workspace.name.as_str(), projects: projects) },
    )
}

#[page("/workspaces/{workspace_id}/projects/new")]
async fn new_workspace_project_page(cx: &Cx) -> Result<impl View> {
    let state = topcoat::context::app_context::<AppState>(cx);
    let Some(hash) = session::token_hash(cx).await? else {
        return Err(topcoat::router::error::redirect("/login").into());
    };
    let user_id = session_user_id(state, &hash)
        .await
        .map_err(|error| internal_server_error(std::io::Error::other(error)))?
        .ok_or_else(|| topcoat::router::error::redirect("/login"))?;
    let id = *path_param::<WorkspaceId>(cx)?;
    if workspace_repo::WorkspaceRepository::new(state.db.clone())
        .find_for_user(user_id, id)
        .await
        .map_err(internal_server_error)?
        .is_none()
    {
        return Err(topcoat::router::error::not_found().into());
    }
    let error = parse_query_params::<BoardPageQuery>(cx)
        .ok()
        .and_then(|query| query.error);
    Ok(view! { create_project_form(workspace_id: id, error: error) })
}

#[page("/projects/{project_id}")]
async fn project_page(cx: &Cx) -> Result<impl View> {
    let state = topcoat::context::app_context::<AppState>(cx);
    let Some(hash) = session::token_hash(cx).await? else {
        return Err(topcoat::router::error::redirect("/login").into());
    };
    let user_id = session_user_id(state, &hash)
        .await
        .map_err(|error| internal_server_error(std::io::Error::other(error)))?
        .ok_or_else(|| topcoat::router::error::redirect("/login"))?;
    let id = *path_param::<ProjectId>(cx)?;
    let project = project_repo::ProjectRepository::new(state.db.clone())
        .find_for_user(user_id, id)
        .await
        .map_err(internal_server_error)?
        .ok_or_else(topcoat::router::error::not_found)?;
    let boards = board_repo::BoardRepository::new(state.db.clone())
        .list_for_project(user_id, id)
        .await
        .map_err(internal_server_error)?
        .into_iter()
        .map(|board| web::projects::view::BoardSummary {
            id: board.id,
            name: board.name,
        })
        .collect();
    Ok(
        view! { web::projects::view::project_view(project_id: project.id, name: project.name.as_str(), boards: boards) },
    )
}

#[page("/projects/{project_id}/boards/new")]
async fn new_project_board_page(cx: &Cx) -> Result<impl View> {
    let state = topcoat::context::app_context::<AppState>(cx);
    let Some(hash) = session::token_hash(cx).await? else {
        return Err(topcoat::router::error::redirect("/login").into());
    };
    let user_id = session_user_id(state, &hash)
        .await
        .map_err(|error| internal_server_error(std::io::Error::other(error)))?
        .ok_or_else(|| topcoat::router::error::redirect("/login"))?;
    let id = *path_param::<ProjectId>(cx)?;
    if project_repo::ProjectRepository::new(state.db.clone())
        .find_for_user(user_id, id)
        .await
        .map_err(internal_server_error)?
        .is_none()
    {
        return Err(topcoat::router::error::not_found().into());
    }
    let error = parse_query_params::<BoardPageQuery>(cx)
        .ok()
        .and_then(|query| query.error);
    Ok(view! { create_board_form(project_id: id, error: error) })
}

#[page("/boards/{board_id}")]
async fn board_page(cx: &Cx) -> Result<impl View> {
    let state = topcoat::context::app_context::<AppState>(cx);
    let Some(hash) = session::token_hash(cx).await? else {
        return Err(topcoat::router::error::redirect("/login").into());
    };
    let user_id = match session_user_id(state, &hash).await {
        Ok(Some(user_id)) => user_id,
        Ok(None) => return Err(topcoat::router::error::redirect("/login").into()),
        Err(error) => {
            tracing::error!("Failed to verify board session: {}", error);
            return Err(internal_server_error(std::io::Error::other(error)).into());
        }
    };
    let id = path_param::<BoardId>(cx)?;
    let board = board_repo::BoardRepository::new(state.db.clone())
        .find_for_user(user_id, *id)
        .await
        .map_err(|error| {
            tracing::error!("Failed to load board {}: {}", id, error);
            internal_server_error(error)
        })?
        .ok_or_else(topcoat::router::error::not_found)?;

    Ok(view! { web::boards::view::board_view(name: board.name.as_str()) })
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

#[route(POST "/projects/{project_id}/boards")]
async fn create_project_board(cx: &Cx, Form(payload): Form<CreateBoardForm>) -> Result<SeeOther> {
    let project_id = *path_param::<ProjectId>(cx)?;
    let name = match validate_board_name(&payload.name) {
        Ok(name) => name,
        Err(BoardNameError::Empty) => {
            return Ok(see_other(&format!(
                "/projects/{project_id}/boards/new?error=empty_name"
            )));
        }
        Err(BoardNameError::TooLong) => {
            return Ok(see_other(&format!(
                "/projects/{project_id}/boards/new?error=name_too_long"
            )));
        }
    };
    let state = topcoat::context::app_context::<AppState>(cx);
    let Some(hash) = session::token_hash(cx).await? else {
        return Ok(see_other("/login"));
    };
    let user_id = session_user_id(state, &hash)
        .await
        .map_err(|error| internal_server_error(std::io::Error::other(error)))?
        .ok_or_else(|| topcoat::router::error::redirect("/login"))?;
    if let Err(error) = board_repo::BoardRepository::new(state.db.clone())
        .create_for_project(user_id, project_id, name)
        .await
    {
        tracing::error!(
            "Failed to create board in project {}: {}",
            project_id,
            error
        );
        return Ok(see_other(&format!(
            "/projects/{project_id}/boards/new?error=storage"
        )));
    }
    Ok(see_other(&format!("/projects/{project_id}")))
}

#[route(POST "/workspaces/new")]
async fn create_workspace(cx: &Cx, Form(payload): Form<CreateWorkspaceForm>) -> Result<SeeOther> {
    let name = match validate_board_name(&payload.name) {
        Ok(name) => name,
        Err(BoardNameError::Empty) => return Ok(see_other("/workspaces/new?error=empty_name")),
        Err(BoardNameError::TooLong) => {
            return Ok(see_other("/workspaces/new?error=name_too_long"));
        }
    };
    let state = topcoat::context::app_context::<AppState>(cx);
    let Some(hash) = session::token_hash(cx).await? else {
        return Ok(see_other("/login"));
    };
    let user_id = session_user_id(state, &hash)
        .await
        .map_err(|error| internal_server_error(std::io::Error::other(error)))?
        .ok_or_else(|| topcoat::router::error::redirect("/login"))?;
    if let Err(error) = workspace_repo::WorkspaceRepository::new(state.db.clone())
        .create_for_user(user_id, name)
        .await
    {
        tracing::error!("Failed to create workspace for user {}: {}", user_id, error);
        return Ok(see_other("/workspaces/new?error=storage"));
    }
    Ok(see_other("/dashboard"))
}

#[route(POST "/workspaces/{workspace_id}/projects")]
async fn create_workspace_project(
    cx: &Cx,
    Form(payload): Form<CreateProjectForm>,
) -> Result<SeeOther> {
    let workspace_id = *path_param::<WorkspaceId>(cx)?;
    let name = match validate_board_name(&payload.name) {
        Ok(name) => name,
        Err(BoardNameError::Empty) => {
            return Ok(see_other(&format!(
                "/workspaces/{workspace_id}/projects/new?error=empty_name"
            )));
        }
        Err(BoardNameError::TooLong) => {
            return Ok(see_other(&format!(
                "/workspaces/{workspace_id}/projects/new?error=name_too_long"
            )));
        }
    };
    let state = topcoat::context::app_context::<AppState>(cx);
    let Some(hash) = session::token_hash(cx).await? else {
        return Ok(see_other("/login"));
    };
    let user_id = session_user_id(state, &hash)
        .await
        .map_err(|error| internal_server_error(std::io::Error::other(error)))?
        .ok_or_else(|| topcoat::router::error::redirect("/login"))?;
    if let Err(error) = project_repo::ProjectRepository::new(state.db.clone())
        .create_for_workspace(user_id, workspace_id, name)
        .await
    {
        tracing::error!(
            "Failed to create project in workspace {}: {}",
            workspace_id,
            error
        );
        return Ok(see_other(&format!(
            "/workspaces/{workspace_id}/projects/new?error=storage"
        )));
    }
    Ok(see_other(&format!("/workspaces/{workspace_id}")))
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
        .page(new_workspace_page)
        .page(workspace_page)
        .page(new_workspace_project_page)
        .page(project_page)
        .page(new_project_board_page)
        .page(board_page)
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
        .route(create_workspace)
        .route(create_workspace_project)
        .route(create_project_board)
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
