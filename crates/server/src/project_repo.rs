use common::error::{AppError, DbError};
use turso::Database;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    pub id: i64,
    pub name: String,
}

pub struct ProjectRepository {
    db: Database,
}

impl ProjectRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn create_for_user(&self, user_id: i64, name: &str) -> Result<(), AppError> {
        let conn = self
            .db
            .connect()
            .map_err(|error| AppError::Database(DbError::Connection(error.to_string())))?;
        conn.execute(
            "INSERT INTO projects (user_id, name) VALUES (?1, ?2)",
            (user_id, name),
        )
        .await
        .map_err(|error| AppError::Database(DbError::Query(error.to_string())))?;
        Ok(())
    }

    pub async fn list_for_user(&self, user_id: i64) -> Result<Vec<Project>, AppError> {
        let conn = self
            .db
            .connect()
            .map_err(|error| AppError::Database(DbError::Connection(error.to_string())))?;
        let mut statement = conn
            .prepare("SELECT id, name FROM projects WHERE user_id = ?1 ORDER BY id DESC")
            .await
            .map_err(|error| AppError::Database(DbError::Query(error.to_string())))?;
        let mut rows = statement
            .query([user_id])
            .await
            .map_err(|error| AppError::Database(DbError::Query(error.to_string())))?;
        let mut projects = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|error| AppError::Database(DbError::Query(error.to_string())))?
        {
            let id = match row
                .get_value(0)
                .map_err(|error| AppError::Database(DbError::Query(error.to_string())))?
            {
                turso::Value::Integer(id) => id,
                value => {
                    return Err(AppError::Database(DbError::Query(format!(
                        "Invalid project ID: {value:?}"
                    ))));
                }
            };
            let name = match row
                .get_value(1)
                .map_err(|error| AppError::Database(DbError::Query(error.to_string())))?
            {
                turso::Value::Text(name) => name,
                value => {
                    return Err(AppError::Database(DbError::Query(format!(
                        "Invalid project name: {value:?}"
                    ))));
                }
            };
            projects.push(Project { id, name });
        }
        Ok(projects)
    }

    pub async fn find_for_user(
        &self,
        user_id: i64,
        project_id: i64,
    ) -> Result<Option<Project>, AppError> {
        let conn = self
            .db
            .connect()
            .map_err(|error| AppError::Database(DbError::Connection(error.to_string())))?;
        let mut statement = conn
            .prepare("SELECT id, name FROM projects WHERE id = ?1 AND user_id = ?2")
            .await
            .map_err(|error| AppError::Database(DbError::Query(error.to_string())))?;
        let mut rows = statement
            .query((project_id, user_id))
            .await
            .map_err(|error| AppError::Database(DbError::Query(error.to_string())))?;
        let Some(row) = rows
            .next()
            .await
            .map_err(|error| AppError::Database(DbError::Query(error.to_string())))?
        else {
            return Ok(None);
        };
        let id = match row
            .get_value(0)
            .map_err(|error| AppError::Database(DbError::Query(error.to_string())))?
        {
            turso::Value::Integer(id) => id,
            value => {
                return Err(AppError::Database(DbError::Query(format!(
                    "Invalid project ID: {value:?}"
                ))));
            }
        };
        let name = match row
            .get_value(1)
            .map_err(|error| AppError::Database(DbError::Query(error.to_string())))?
        {
            turso::Value::Text(name) => name,
            value => {
                return Err(AppError::Database(DbError::Query(format!(
                    "Invalid project name: {value:?}"
                ))));
            }
        };
        Ok(Some(Project { id, name }))
    }
}
