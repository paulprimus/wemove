use common::error::{AppError, DbError};
use turso::Database;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceMember {
    pub user_id: i64,
    pub role: String,
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
        conn.execute(
            "INSERT INTO workspace_members (workspace_id, user_id, role)
             VALUES (last_insert_rowid(), ?1, 'admin')",
            [user_id],
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
            .prepare(
                "SELECT workspaces.id, workspaces.name
                 FROM workspaces
                 INNER JOIN workspace_members ON workspace_members.workspace_id = workspaces.id
                 WHERE workspace_members.user_id = ?1
                 ORDER BY workspaces.id DESC",
            )
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
            .prepare(
                "SELECT workspaces.id, workspaces.name
                 FROM workspaces
                 INNER JOIN workspace_members ON workspace_members.workspace_id = workspaces.id
                 WHERE workspaces.id = ?1 AND workspace_members.user_id = ?2",
            )
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

    pub async fn add_member(
        &self,
        workspace_id: i64,
        user_id: i64,
        role: &str,
    ) -> Result<(), AppError> {
        let conn = self
            .db
            .connect()
            .map_err(|e| AppError::Database(DbError::Connection(e.to_string())))?;
        conn.execute(
            "INSERT INTO workspace_members (workspace_id, user_id, role) VALUES (?1, ?2, ?3)",
            (workspace_id, user_id, role),
        )
        .await
        .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;
        Ok(())
    }

    pub async fn list_members(&self, workspace_id: i64) -> Result<Vec<WorkspaceMember>, AppError> {
        let conn = self
            .db
            .connect()
            .map_err(|e| AppError::Database(DbError::Connection(e.to_string())))?;
        let mut statement = conn
            .prepare(
                "SELECT user_id, role FROM workspace_members
                 WHERE workspace_id = ?1 ORDER BY user_id",
            )
            .await
            .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;
        let mut rows = statement
            .query([workspace_id])
            .await
            .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;
        let mut members = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?
        {
            let user_id = match row
                .get_value(0)
                .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?
            {
                turso::Value::Integer(value) => value,
                value => {
                    return Err(AppError::Database(DbError::Query(format!(
                        "Invalid workspace member user ID: {value:?}"
                    ))));
                }
            };
            let role = match row
                .get_value(1)
                .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?
            {
                turso::Value::Text(value) => value,
                value => {
                    return Err(AppError::Database(DbError::Query(format!(
                        "Invalid workspace member role: {value:?}"
                    ))));
                }
            };
            members.push(WorkspaceMember { user_id, role });
        }
        Ok(members)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, time::SystemTime};

    use super::*;
    use turso::Builder;

    #[tokio::test]
    async fn workspace_membership_links_users_with_roles_and_controls_workspace_lookup() {
        let suffix = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("system clock should be after the Unix epoch")
            .as_nanos();
        let db_path = std::env::temp_dir().join(format!("wemove-workspaces-{suffix}.db"));
        let db_path_string = db_path.to_string_lossy();
        let db = Builder::new_local(db_path_string.as_ref())
            .build()
            .await
            .expect("test database should be created");
        let conn = db.connect().expect("test database should connect");
        conn.execute(
            "CREATE TABLE users (id INTEGER PRIMARY KEY, email TEXT NOT NULL)",
            (),
        )
        .await
        .expect("users table should be created");
        conn.execute(
            "CREATE TABLE workspaces (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                name TEXT NOT NULL
            )",
            (),
        )
        .await
        .expect("workspaces table should be created");
        conn.execute(
            "CREATE TABLE workspace_members (
                workspace_id INTEGER NOT NULL,
                user_id INTEGER NOT NULL,
                role TEXT NOT NULL CHECK (role IN ('admin', 'member')),
                PRIMARY KEY (workspace_id, user_id)
            )",
            (),
        )
        .await
        .expect("workspace members table should be created");
        conn.execute("INSERT INTO users (id, email) VALUES (1, 'owner@example.com'), (2, 'member@example.com')", ())
            .await
            .expect("test users should be inserted");

        let repository = WorkspaceRepository::new(db.clone());
        repository
            .create_for_user(1, "Acme")
            .await
            .expect("workspace should be created for its owner");
        repository
            .add_member(1, 2, "member")
            .await
            .expect("user should be added to the workspace");

        assert_eq!(
            repository
                .list_for_user(2)
                .await
                .expect("membership should list workspace"),
            [Workspace {
                id: 1,
                name: "Acme".to_owned()
            }]
        );
        assert_eq!(
            repository
                .find_for_user(2, 1)
                .await
                .expect("member lookup should succeed"),
            Some(Workspace {
                id: 1,
                name: "Acme".to_owned()
            })
        );
        assert_eq!(
            repository
                .list_members(1)
                .await
                .expect("members should load"),
            [
                WorkspaceMember {
                    user_id: 1,
                    role: "admin".to_owned()
                },
                WorkspaceMember {
                    user_id: 2,
                    role: "member".to_owned()
                },
            ]
        );
        assert!(
            repository
                .list_for_user(3)
                .await
                .expect("unknown user has no workspaces")
                .is_empty()
        );
        assert_eq!(
            repository
                .find_for_user(3, 1)
                .await
                .expect("non-member lookup succeeds"),
            None
        );

        drop(repository);
        drop(conn);
        drop(db);
        fs::remove_file(db_path).expect("test database should be removed");
    }
}
