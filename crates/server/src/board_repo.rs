use common::error::{AppError, DbError};
use turso::Database;

pub struct BoardRepository {
    db: Database,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    pub id: i64,
    pub name: String,
}

impl BoardRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn create_for_user(&self, user_id: i64, name: &str) -> Result<(), AppError> {
        let conn = self
            .db
            .connect()
            .map_err(|error| AppError::Database(DbError::Connection(error.to_string())))?;

        conn.execute(
            "INSERT INTO boards (user_id, name) VALUES (?1, ?2)",
            (user_id, name),
        )
        .await
        .map_err(|error| AppError::Database(DbError::Query(error.to_string())))?;

        Ok(())
    }

    pub async fn create_for_project(
        &self,
        user_id: i64,
        project_id: i64,
        name: &str,
    ) -> Result<(), AppError> {
        let conn = self
            .db
            .connect()
            .map_err(|error| AppError::Database(DbError::Connection(error.to_string())))?;
        conn.execute(
            "INSERT INTO boards (user_id, project_id, name) SELECT ?1, id, ?3 FROM projects WHERE id = ?2 AND user_id = ?1",
            (user_id, project_id, name),
        )
        .await
        .map_err(|error| AppError::Database(DbError::Query(error.to_string())))?;
        Ok(())
    }

    pub async fn list_for_user(&self, user_id: i64) -> Result<Vec<Board>, AppError> {
        let conn = self
            .db
            .connect()
            .map_err(|error| AppError::Database(DbError::Connection(error.to_string())))?;
        let mut statement = conn
            .prepare("SELECT id, name FROM boards WHERE user_id = ?1 ORDER BY id DESC")
            .await
            .map_err(|error| AppError::Database(DbError::Query(error.to_string())))?;
        let mut rows = statement
            .query([user_id])
            .await
            .map_err(|error| AppError::Database(DbError::Query(error.to_string())))?;
        let mut boards = Vec::new();

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
                        "Invalid board ID value: {value:?}"
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
                        "Invalid board name value: {value:?}"
                    ))));
                }
            };
            boards.push(Board { id, name });
        }

        Ok(boards)
    }

    pub async fn find_for_user(
        &self,
        user_id: i64,
        board_id: i64,
    ) -> Result<Option<Board>, AppError> {
        let conn = self
            .db
            .connect()
            .map_err(|error| AppError::Database(DbError::Connection(error.to_string())))?;
        let mut statement = conn
            .prepare("SELECT id, name FROM boards WHERE id = ?1 AND user_id = ?2")
            .await
            .map_err(|error| AppError::Database(DbError::Query(error.to_string())))?;
        let mut rows = statement
            .query((board_id, user_id))
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
                    "Invalid board ID value: {value:?}"
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
                    "Invalid board name value: {value:?}"
                ))));
            }
        };

        Ok(Some(Board { id, name }))
    }

    pub async fn list_for_project(
        &self,
        user_id: i64,
        project_id: i64,
    ) -> Result<Vec<Board>, AppError> {
        let conn = self
            .db
            .connect()
            .map_err(|error| AppError::Database(DbError::Connection(error.to_string())))?;
        let mut statement = conn.prepare("SELECT id, name FROM boards WHERE user_id = ?1 AND project_id = ?2 ORDER BY id DESC").await
            .map_err(|error| AppError::Database(DbError::Query(error.to_string())))?;
        let mut rows = statement
            .query((user_id, project_id))
            .await
            .map_err(|error| AppError::Database(DbError::Query(error.to_string())))?;
        let mut boards = Vec::new();
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
                        "Invalid board ID value: {value:?}"
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
                        "Invalid board name value: {value:?}"
                    ))));
                }
            };
            boards.push(Board { id, name });
        }
        Ok(boards)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, time::SystemTime};

    use super::*;
    use turso::Builder;

    #[tokio::test]
    async fn board_creation_is_persisted_and_scoped_to_its_owner() {
        let suffix = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("system clock should be after the Unix epoch")
            .as_nanos();
        let db_path = std::env::temp_dir().join(format!("wemove-boards-{suffix}.db"));
        let db_path_string = db_path.to_string_lossy();
        let db = Builder::new_local(db_path_string.as_ref())
            .build()
            .await
            .expect("test database should be created");
        let conn = db.connect().expect("test database should connect");
        conn.execute(
            "CREATE TABLE boards (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                name TEXT NOT NULL
            )",
            (),
        )
        .await
        .expect("test boards table should be created");

        let repository = BoardRepository::new(db.clone());
        repository
            .create_for_user(1, "Roadmap")
            .await
            .expect("board should be created");
        repository
            .create_for_user(2, "Private board")
            .await
            .expect("second user's board should be created");

        assert_eq!(
            repository
                .list_for_user(1)
                .await
                .expect("boards should load"),
            [Board {
                id: 1,
                name: "Roadmap".to_owned(),
            }]
        );
        assert_eq!(
            repository
                .list_for_user(2)
                .await
                .expect("boards should load"),
            [Board {
                id: 2,
                name: "Private board".to_owned(),
            }]
        );
        assert_eq!(
            repository
                .find_for_user(1, 1)
                .await
                .expect("board should load"),
            Some(Board {
                id: 1,
                name: "Roadmap".to_owned(),
            })
        );
        assert_eq!(
            repository
                .find_for_user(2, 1)
                .await
                .expect("foreign board lookup should succeed"),
            None
        );

        drop(repository);
        drop(conn);
        drop(db);
        fs::remove_file(db_path).expect("test database should be removed");
    }
}
