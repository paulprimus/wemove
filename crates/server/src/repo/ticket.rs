use common::error::{AppError, DbError};
use turso::Database;

pub struct TicketRepository {
    db: Database,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ticket {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub status: String,
}

fn query_error(error: impl ToString) -> AppError {
    AppError::Database(DbError::Query(error.to_string()))
}

impl TicketRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Creates a ticket if the user may edit the board's project. Returns whether it was created.
    pub async fn create_for_board(
        &self,
        user_id: i64,
        board_id: i64,
        title: &str,
        description: &str,
    ) -> Result<bool, AppError> {
        let conn = self
            .db
            .connect()
            .map_err(|error| AppError::Database(DbError::Connection(error.to_string())))?;
        let created = conn
            .execute(
                "INSERT INTO tickets (board_id, created_by, title, description)
                 SELECT boards.id, ?1, ?3, ?4 FROM boards
                 INNER JOIN project_members ON project_members.project_id = boards.project_id
                 WHERE boards.id = ?2 AND project_members.user_id = ?1
                   AND project_members.role IN ('owner', 'admin', 'member')",
                (user_id, board_id, title, description),
            )
            .await
            .map_err(query_error)?;
        Ok(created > 0)
    }

    pub async fn list_for_board(
        &self,
        user_id: i64,
        board_id: i64,
    ) -> Result<Vec<Ticket>, AppError> {
        let conn = self
            .db
            .connect()
            .map_err(|error| AppError::Database(DbError::Connection(error.to_string())))?;
        let mut statement = conn
            .prepare(
                "SELECT tickets.id, tickets.title, tickets.description, tickets.status
                 FROM tickets
                 INNER JOIN boards ON boards.id = tickets.board_id
                 INNER JOIN project_members ON project_members.project_id = boards.project_id
                 WHERE tickets.board_id = ?1 AND project_members.user_id = ?2
                 ORDER BY tickets.id ASC",
            )
            .await
            .map_err(query_error)?;
        let mut rows = statement
            .query((board_id, user_id))
            .await
            .map_err(query_error)?;
        let mut tickets = Vec::new();
        while let Some(row) = rows.next().await.map_err(query_error)? {
            let (
                turso::Value::Integer(id),
                turso::Value::Text(title),
                turso::Value::Text(description),
                turso::Value::Text(status),
            ) = (
                row.get_value(0).map_err(query_error)?,
                row.get_value(1).map_err(query_error)?,
                row.get_value(2).map_err(query_error)?,
                row.get_value(3).map_err(query_error)?,
            )
            else {
                return Err(query_error("Invalid ticket row"));
            };
            tickets.push(Ticket {
                id,
                title,
                description,
                status,
            });
        }
        Ok(tickets)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, time::SystemTime};

    use super::*;
    use turso::Builder;

    #[tokio::test]
    async fn tickets_are_scoped_to_project_members_with_edit_rights() {
        let suffix = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("system clock should be after the Unix epoch")
            .as_nanos();
        let db_path = std::env::temp_dir().join(format!("wemove-tickets-{suffix}.db"));
        let db = Builder::new_local(db_path.to_string_lossy().as_ref())
            .build()
            .await
            .expect("test database should be created");
        let conn = db.connect().expect("test database should connect");
        for sql in [
            "CREATE TABLE project_members (project_id INTEGER NOT NULL, user_id INTEGER NOT NULL, role TEXT NOT NULL)",
            "CREATE TABLE boards (id INTEGER PRIMARY KEY, project_id INTEGER NOT NULL)",
            "CREATE TABLE tickets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                board_id INTEGER NOT NULL,
                created_by INTEGER NOT NULL,
                title TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                status TEXT NOT NULL DEFAULT 'todo'
            )",
            "INSERT INTO boards (id, project_id) VALUES (1, 1)",
            "INSERT INTO project_members VALUES (1, 1, 'member'), (1, 2, 'viewer')",
        ] {
            conn.execute(sql, ())
                .await
                .expect("test setup should succeed");
        }

        let repository = TicketRepository::new(db.clone());
        assert!(
            repository
                .create_for_board(1, 1, "Task", "Details")
                .await
                .unwrap()
        );
        assert!(!repository.create_for_board(2, 1, "Nope", "").await.unwrap());
        assert!(!repository.create_for_board(3, 1, "Nope", "").await.unwrap());

        let expected = [Ticket {
            id: 1,
            title: "Task".to_owned(),
            description: "Details".to_owned(),
            status: "todo".to_owned(),
        }];
        assert_eq!(repository.list_for_board(1, 1).await.unwrap(), expected);
        assert_eq!(repository.list_for_board(2, 1).await.unwrap(), expected);
        assert!(repository.list_for_board(3, 1).await.unwrap().is_empty());

        drop(repository);
        drop(conn);
        drop(db);
        fs::remove_file(db_path).expect("test database should be removed");
    }
}
