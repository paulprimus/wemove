use common::error::{AppError, DbError};
use turso::Builder;

#[derive(Clone)]
pub struct AppState {
    pub app_name: String,
    pub db: turso::Database,
}

impl AppState {
    pub async fn new() -> Result<Self, AppError> {
        let db = Builder::new_local("db/wemove.db")
            .build()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        run_migrations(&db).await?;

        Ok(Self {
            app_name: "wemove".to_string(),
            db,
        })
    }
}

async fn run_migrations(db: &turso::Database) -> Result<(), AppError> {
    let conn = db.connect().map_err(|e| AppError::Internal(e.to_string()))?;

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

    tracing::info!("Database migrations completed");
    Ok(())
}