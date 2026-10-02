use common::error::{AppError, DbError};
use turso::Database;

pub struct BoardRepository {
    db: Database,
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

    pub async fn list_for_user(&self, user_id: i64) -> Result<Vec<String>, AppError> {
        let conn = self
            .db
            .connect()
            .map_err(|error| AppError::Database(DbError::Connection(error.to_string())))?;
        let mut statement = conn
            .prepare("SELECT name FROM boards WHERE user_id = ?1 ORDER BY id DESC")
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
            match row
                .get_value(0)
                .map_err(|error| AppError::Database(DbError::Query(error.to_string())))?
            {
                turso::Value::Text(name) => boards.push(name),
                value => {
                    return Err(AppError::Database(DbError::Query(format!(
                        "Invalid board name value: {value:?}"
                    ))));
                }
            }
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
            ["Roadmap"]
        );
        assert_eq!(
            repository
                .list_for_user(2)
                .await
                .expect("boards should load"),
            ["Private board"]
        );

        drop(repository);
        drop(conn);
        drop(db);
        fs::remove_file(db_path).expect("test database should be removed");
    }
}
