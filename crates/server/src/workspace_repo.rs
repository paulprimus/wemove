use common::error::{AppError, DbError};
use turso::Database;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    pub id: i64,
    pub name: String,
}

pub struct WorkspaceRepository {
    db: Database,
}

impl WorkspaceRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn create_for_user(&self, user_id: i64, name: &str) -> Result<(), AppError> {
        let conn = self
            .db
            .connect()
            .map_err(|e| AppError::Database(DbError::Connection(e.to_string())))?;
        conn.execute(
            "INSERT INTO workspaces (user_id, name) VALUES (?1, ?2)",
            (user_id, name),
        )
        .await
        .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;
        Ok(())
    }

    pub async fn list_for_user(&self, user_id: i64) -> Result<Vec<Workspace>, AppError> {
        let conn = self
            .db
            .connect()
            .map_err(|e| AppError::Database(DbError::Connection(e.to_string())))?;
        let mut statement = conn
            .prepare("SELECT id, name FROM workspaces WHERE user_id = ?1 ORDER BY id DESC")
            .await
            .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;
        let mut rows = statement
            .query([user_id])
            .await
            .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;
        let mut workspaces = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?
        {
            let id = match row
                .get_value(0)
                .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?
            {
                turso::Value::Integer(value) => value,
                value => {
                    return Err(AppError::Database(DbError::Query(format!(
                        "Invalid workspace ID: {value:?}"
                    ))));
                }
            };
            let name = match row
                .get_value(1)
                .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?
            {
                turso::Value::Text(value) => value,
                value => {
                    return Err(AppError::Database(DbError::Query(format!(
                        "Invalid workspace name: {value:?}"
                    ))));
                }
            };
            workspaces.push(Workspace { id, name });
        }
        Ok(workspaces)
    }

    pub async fn find_for_user(
        &self,
        user_id: i64,
        workspace_id: i64,
    ) -> Result<Option<Workspace>, AppError> {
        let conn = self
            .db
            .connect()
            .map_err(|e| AppError::Database(DbError::Connection(e.to_string())))?;
        let mut statement = conn
            .prepare("SELECT id, name FROM workspaces WHERE id = ?1 AND user_id = ?2")
            .await
            .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;
        let mut rows = statement
            .query((workspace_id, user_id))
            .await
            .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;
        let Some(row) = rows
            .next()
            .await
            .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?
        else {
            return Ok(None);
        };
        let id = match row
            .get_value(0)
            .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?
        {
            turso::Value::Integer(value) => value,
            value => {
                return Err(AppError::Database(DbError::Query(format!(
                    "Invalid workspace ID: {value:?}"
                ))));
            }
        };
        let name = match row
            .get_value(1)
            .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?
        {
            turso::Value::Text(value) => value,
            value => {
                return Err(AppError::Database(DbError::Query(format!(
                    "Invalid workspace name: {value:?}"
                ))));
            }
        };
        Ok(Some(Workspace { id, name }))
    }
}
