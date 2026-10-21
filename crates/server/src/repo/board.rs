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
            "INSERT INTO boards (user_id, project_id, name)
             SELECT ?1, projects.id, ?3 FROM projects
             INNER JOIN project_members ON project_members.project_id = projects.id
             WHERE projects.id = ?2 AND project_members.user_id = ?1
               AND project_members.role IN ('owner', 'admin', 'member')",
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
            .prepare(
                "SELECT boards.id, boards.name FROM boards
                 INNER JOIN project_members ON project_members.project_id = boards.project_id
                 WHERE boards.id = ?1 AND project_members.user_id = ?2",
            )
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

    pub async fn can_edit(&self, user_id: i64, board_id: i64) -> Result<bool, AppError> {
        let conn = self
            .db
            .connect()
            .map_err(|error| AppError::Database(DbError::Connection(error.to_string())))?;
        let mut statement = conn
            .prepare(
                "SELECT 1 FROM boards
                 INNER JOIN project_members ON project_members.project_id = boards.project_id
                 WHERE boards.id = ?1 AND project_members.user_id = ?2
                   AND project_members.role IN ('owner', 'admin', 'member')",
            )
            .await
            .map_err(|error| AppError::Database(DbError::Query(error.to_string())))?;
        let mut rows = statement
            .query((board_id, user_id))
            .await
            .map_err(|error| AppError::Database(DbError::Query(error.to_string())))?;
        Ok(rows
            .next()
            .await
            .map_err(|error| AppError::Database(DbError::Query(error.to_string())))?
            .is_some())
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
        let mut statement = conn.prepare("SELECT boards.id, boards.name FROM boards INNER JOIN project_members ON project_members.project_id = boards.project_id WHERE project_members.user_id = ?1 AND boards.project_id = ?2 ORDER BY boards.id DESC").await
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
            "CREATE TABLE projects (
                id INTEGER PRIMARY KEY,
                user_id INTEGER NOT NULL,
                name TEXT NOT NULL
            )",
            (),
        )
        .await
        .expect("test projects table should be created");
        conn.execute(
            "CREATE TABLE project_members (
                project_id INTEGER NOT NULL,
                user_id INTEGER NOT NULL,
                role TEXT NOT NULL,
                PRIMARY KEY (project_id, user_id)
            )",
            (),
        )
        .await
        .expect("test project members table should be created");
        conn.execute(
            "CREATE TABLE boards (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                project_id INTEGER,
                name TEXT NOT NULL
            )",
            (),
        )
        .await
        .expect("test boards table should be created");
        conn.execute(
            "INSERT INTO projects (id, user_id, name) VALUES (1, 1, 'First'), (2, 2, 'Second')",
            (),
        )
        .await
        .expect("test projects should be inserted");
        conn.execute(
            "INSERT INTO project_members (project_id, user_id, role)
             VALUES (1, 1, 'owner'), (2, 2, 'owner')",
            (),
        )
        .await
        .expect("project owners should be inserted");

        let repository = BoardRepository::new(db.clone());
        repository
            .create_for_project(1, 1, "Roadmap")
            .await
            .expect("board should be created");
        repository
            .create_for_project(2, 2, "Private board")
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
