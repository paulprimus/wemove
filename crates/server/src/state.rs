use common::error::{AppError, DbError};
use serde::{Deserialize, Serialize};
use turso::{Builder, Connection};

#[derive(Debug, Clone)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub name: String,
}

#[derive(Clone)]
pub struct AppState {
    pub app_name: String,
    pub db: turso::Database,
    pub jwt_secret: Vec<u8>,
    pub token_expiry_secs: u64,
}

impl AppState {
    pub async fn new(jwt_secret: Vec<u8>, token_expiry_secs: u64) -> Result<Self, AppError> {
        let db = Builder::new_local("db/wemove.db")
            .build()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        run_migrations(&db).await?;

        Ok(Self {
            app_name: "wemove".to_string(),
            db,
            jwt_secret,
            token_expiry_secs,
        })
    }

    pub fn create_access_token(&self, client_id: &str, scope: &str) -> Result<String, AppError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let claims = JwtClaims {
            sub: client_id.to_string(),
            scope: scope.to_string(),
            exp: now + self.token_expiry_secs,
            iat: now,
        };

        jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(&self.jwt_secret),
        )
        .map_err(|e| AppError::Internal(format!("JWT creation failed: {e}")))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub scope: String,
    pub exp: u64,
    pub iat: u64,
}

async fn run_migrations(db: &turso::Database) -> Result<(), AppError> {
    let conn = db
        .connect()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    create_user_table(&conn).await?;

    create_session_table(&conn).await?;

    create_board_table(&conn).await?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_boards_user_id ON boards(user_id)",
        (),
    )
    .await
    .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;

    tracing::info!("Database migrations completed");
    Ok(())
}

async fn create_board_table(conn: &Connection) -> Result<(), AppError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS boards (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )",
        (),
    )
    .await
    .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;
    Ok(())
}

async fn create_user_table(conn: &Connection) -> Result<(), AppError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            email TEXT UNIQUE NOT NULL,
            name TEXT NOT NULL,
            password_hash TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        (),
    )
    .await
    .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;
    Ok(())
}

async fn create_session_table(conn: &Connection) -> Result<(), AppError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            token_hash TEXT PRIMARY KEY,
            user_id INTEGER NOT NULL,
            expires_at INTEGER NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )",
        (),
    )
    .await
    .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id)",
        (),
    )
    .await
    .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;

    Ok(())
}
