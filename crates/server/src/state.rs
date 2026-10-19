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

    create_workspace_table(&conn).await?;

    create_workspace_members_table(&conn).await?;

    create_project_table(&conn).await?;

    create_project_members_table(&conn).await?;

    conn.execute(
        "INSERT OR IGNORE INTO project_members (project_id, user_id, role)
         SELECT id, user_id, 'owner' FROM projects",
        (),
    )
    .await
    .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;

    create_board_table(&conn).await?;

    // Legacy boards tables may predate project associations.
    if let Err(error) = conn
        .execute("ALTER TABLE boards ADD COLUMN project_id INTEGER", ())
        .await
    {
        let message = error.to_string();
        if !message.contains("duplicate column") {
            return Err(AppError::Database(DbError::Query(message)));
        }
    }

    if let Err(error) = conn
        .execute("ALTER TABLE projects ADD COLUMN workspace_id INTEGER", ())
        .await
    {
        let message = error.to_string();
        if !message.contains("duplicate column") {
            return Err(AppError::Database(DbError::Query(message)));
        }
    }

    // Put records from older versions into a personal workspace so existing
    // projects and boards remain visible after the hierarchy migration.
    conn.execute(
        "INSERT INTO workspaces (user_id, name)
         SELECT DISTINCT user_id, 'Personal workspace'
         FROM projects
         WHERE workspace_id IS NULL
           AND user_id NOT IN (SELECT user_id FROM workspaces)",
        (),
    )
    .await
    .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;
    // Preserve ownership for existing workspaces and for personal workspaces
    // created above during the hierarchy migration.
    conn.execute(
        "INSERT OR IGNORE INTO workspace_members (workspace_id, user_id, role)
         SELECT id, user_id, 'admin' FROM workspaces",
        (),
    )
    .await
    .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;
    conn.execute(
        "UPDATE projects
         SET workspace_id = (SELECT id FROM workspaces WHERE workspaces.user_id = projects.user_id ORDER BY id LIMIT 1)
         WHERE workspace_id IS NULL",
        (),
    )
    .await
    .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;
    conn.execute(
        "UPDATE boards
         SET project_id = (SELECT id FROM projects WHERE projects.user_id = boards.user_id ORDER BY id LIMIT 1)
         WHERE project_id IS NULL",
        (),
    )
    .await
    .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_boards_user_id ON boards(user_id)",
        (),
    )
    .await
    .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;

    tracing::info!("Database migrations completed");
    Ok(())
}

async fn create_workspace_table(conn: &Connection) -> Result<(), AppError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS workspaces (
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
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_workspaces_user_id ON workspaces(user_id)",
        (),
    )
    .await
    .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;
    Ok(())
}

async fn create_workspace_members_table(conn: &Connection) -> Result<(), AppError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS workspace_members (
            workspace_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,
            role TEXT NOT NULL CHECK (role IN ('admin', 'member')),
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (workspace_id, user_id),
            FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )",
        (),
    )
    .await
    .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_workspace_members_user_id ON workspace_members(user_id)",
        (),
    )
    .await
    .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;
    Ok(())
}

async fn create_project_table(conn: &Connection) -> Result<(), AppError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            workspace_id INTEGER,
            name TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE
        )",
        (),
    )
    .await
    .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_projects_user_id ON projects(user_id)",
        (),
    )
    .await
    .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;
    Ok(())
}

async fn create_board_table(conn: &Connection) -> Result<(), AppError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS boards (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            project_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
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

async fn create_project_members_table(conn: &Connection) -> Result<(), AppError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS project_members (
            project_id integer not null,
            user_id INTEGER NOT NULL,
            role TEXT NOT NULL CHECK (role IN ('owner', 'admin', 'member', 'viewer')),
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (project_id, user_id),
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )",
        (),
    )
    .await
    .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_project_members_user_id ON project_members(user_id)",
        (),
    )
    .await
    .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{fs, time::SystemTime};

    use super::*;

    #[tokio::test]
    async fn migration_adds_project_id_before_backfilling_legacy_boards() {
        let suffix = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("system clock should be after the Unix epoch")
            .as_nanos();
        let db_path = std::env::temp_dir().join(format!("wemove-legacy-boards-{suffix}.db"));
        let db_path_string = db_path.to_string_lossy();
        let db = Builder::new_local(db_path_string.as_ref())
            .build()
            .await
            .expect("test database should be created");
        let conn = db.connect().expect("test database should connect");

        conn.execute(
            "CREATE TABLE users (
                id INTEGER PRIMARY KEY,
                email TEXT UNIQUE NOT NULL,
                name TEXT NOT NULL,
                password_hash TEXT NOT NULL
            )",
            (),
        )
        .await
        .expect("legacy users table should be created");
        conn.execute(
            "CREATE TABLE projects (
                id INTEGER PRIMARY KEY,
                user_id INTEGER NOT NULL,
                name TEXT NOT NULL
            )",
            (),
        )
        .await
        .expect("legacy projects table should be created");
        conn.execute(
            "CREATE TABLE boards (
                id INTEGER PRIMARY KEY,
                user_id INTEGER NOT NULL,
                name TEXT NOT NULL
            )",
            (),
        )
        .await
        .expect("legacy boards table should be created");
        conn.execute(
            "INSERT INTO users (id, email, name, password_hash)
             VALUES (1, 'user@example.com', 'User', 'hash')",
            (),
        )
        .await
        .expect("legacy user should be inserted");
        conn.execute(
            "INSERT INTO projects (id, user_id, name) VALUES (7, 1, 'Project')",
            (),
        )
        .await
        .expect("legacy project should be inserted");
        conn.execute(
            "INSERT INTO boards (id, user_id, name) VALUES (11, 1, 'Board')",
            (),
        )
        .await
        .expect("legacy board should be inserted");

        run_migrations(&db)
            .await
            .expect("legacy database migrations should succeed");

        let mut statement = conn
            .prepare("SELECT project_id FROM boards WHERE id = 11")
            .await
            .expect("migrated board query should prepare");
        let mut rows = statement
            .query(())
            .await
            .expect("migrated board query should execute");
        let row = rows
            .next()
            .await
            .expect("migrated board query should succeed")
            .expect("migrated board should exist");
        assert_eq!(
            row.get_value(0)
                .expect("migrated project ID should be readable"),
            turso::Value::Integer(7)
        );

        drop(rows);
        drop(statement);
        drop(conn);
        drop(db);
        fs::remove_file(&db_path).expect("test database should be removed");
    }
}
