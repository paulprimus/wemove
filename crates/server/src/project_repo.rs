use common::error::{AppError, DbError};
use turso::Database;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectMember {
    pub user_id: i64,
    pub name: String,
    pub email: String,
    pub role: String,
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
        conn.execute(
            "INSERT INTO project_members (project_id, user_id, role)
             VALUES (last_insert_rowid(), ?1, 'owner')",
            [user_id],
        )
        .await
        .map_err(|error| AppError::Database(DbError::Query(error.to_string())))?;
        Ok(())
    }

    pub async fn create_for_workspace(
        &self,
        user_id: i64,
        workspace_id: i64,
        name: &str,
    ) -> Result<(), AppError> {
        let conn = self
            .db
            .connect()
            .map_err(|e| AppError::Database(DbError::Connection(e.to_string())))?;
        conn.execute(
            "INSERT INTO projects (user_id, workspace_id, name) SELECT ?1, id, ?3 FROM workspaces WHERE id = ?2 AND user_id = ?1",
            (user_id, workspace_id, name),
        )
        .await
        .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;
        conn.execute(
            "INSERT INTO project_members (project_id, user_id, role)
             VALUES (last_insert_rowid(), ?1, 'owner')",
            [user_id],
        )
        .await
        .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;
        Ok(())
    }

    pub async fn list_for_user(&self, user_id: i64) -> Result<Vec<Project>, AppError> {
        let conn = self
            .db
            .connect()
            .map_err(|error| AppError::Database(DbError::Connection(error.to_string())))?;
        let mut statement = conn
            .prepare(
                "SELECT projects.id, projects.name FROM projects
                 INNER JOIN project_members ON project_members.project_id = projects.id
                 WHERE project_members.user_id = ?1 ORDER BY projects.id DESC",
            )
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

    pub async fn list_for_workspace(
        &self,
        user_id: i64,
        workspace_id: i64,
    ) -> Result<Vec<Project>, AppError> {
        let conn = self
            .db
            .connect()
            .map_err(|e| AppError::Database(DbError::Connection(e.to_string())))?;
        let mut statement = conn.prepare("SELECT projects.id, projects.name FROM projects INNER JOIN project_members ON project_members.project_id = projects.id WHERE project_members.user_id = ?1 AND projects.workspace_id = ?2 ORDER BY projects.id DESC").await
            .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;
        let mut rows = statement
            .query((user_id, workspace_id))
            .await
            .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;
        let mut projects = Vec::new();
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
                        "Invalid project ID: {value:?}"
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
            .prepare(
                "SELECT projects.id, projects.name FROM projects
                 INNER JOIN project_members ON project_members.project_id = projects.id
                 WHERE projects.id = ?1 AND project_members.user_id = ?2",
            )
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

    pub async fn role_for_user(
        &self,
        project_id: i64,
        user_id: i64,
    ) -> Result<Option<String>, AppError> {
        let conn = self
            .db
            .connect()
            .map_err(|e| AppError::Database(DbError::Connection(e.to_string())))?;
        let mut statement = conn
            .prepare("SELECT role FROM project_members WHERE project_id = ?1 AND user_id = ?2")
            .await
            .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;
        let mut rows = statement
            .query((project_id, user_id))
            .await
            .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;
        let Some(row) = rows
            .next()
            .await
            .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?
        else {
            return Ok(None);
        };
        match row
            .get_value(0)
            .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?
        {
            turso::Value::Text(role) => Ok(Some(role)),
            value => Err(AppError::Database(DbError::Query(format!(
                "Invalid project member role: {value:?}"
            )))),
        }
    }

    pub async fn list_members(&self, project_id: i64) -> Result<Vec<ProjectMember>, AppError> {
        let conn = self
            .db
            .connect()
            .map_err(|e| AppError::Database(DbError::Connection(e.to_string())))?;
        let mut statement = conn
            .prepare(
                "SELECT users.id, users.name, users.email, project_members.role
                 FROM project_members
                 INNER JOIN users ON users.id = project_members.user_id
                 WHERE project_members.project_id = ?1
                 ORDER BY CASE project_members.role WHEN 'owner' THEN 0 ELSE 1 END, users.name",
            )
            .await
            .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;
        let mut rows = statement
            .query([project_id])
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
                turso::Value::Integer(id) => id,
                value => {
                    return Err(AppError::Database(DbError::Query(format!(
                        "Invalid member user ID: {value:?}"
                    ))));
                }
            };
            let name = match row
                .get_value(1)
                .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?
            {
                turso::Value::Text(name) => name,
                value => {
                    return Err(AppError::Database(DbError::Query(format!(
                        "Invalid member name: {value:?}"
                    ))));
                }
            };
            let email = match row
                .get_value(2)
                .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?
            {
                turso::Value::Text(email) => email,
                value => {
                    return Err(AppError::Database(DbError::Query(format!(
                        "Invalid member email: {value:?}"
                    ))));
                }
            };
            let role = match row
                .get_value(3)
                .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?
            {
                turso::Value::Text(role) => role,
                value => {
                    return Err(AppError::Database(DbError::Query(format!(
                        "Invalid member role: {value:?}"
                    ))));
                }
            };
            members.push(ProjectMember {
                user_id,
                name,
                email,
                role,
            });
        }
        Ok(members)
    }

    pub async fn ensure_workspace_member(
        &self,
        project_id: i64,
        user_id: i64,
    ) -> Result<(), AppError> {
        let conn = self
            .db
            .connect()
            .map_err(|e| AppError::Database(DbError::Connection(e.to_string())))?;
        conn.execute(
            "INSERT OR IGNORE INTO workspace_members (workspace_id, user_id, role)
             SELECT workspace_id, ?2, 'member' FROM projects WHERE id = ?1",
            (project_id, user_id),
        )
        .await
        .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;
        Ok(())
    }

    pub async fn add_member(
        &self,
        project_id: i64,
        user_id: i64,
        role: &str,
    ) -> Result<(), AppError> {
        let conn = self
            .db
            .connect()
            .map_err(|e| AppError::Database(DbError::Connection(e.to_string())))?;
        conn.execute(
            "INSERT INTO project_members (project_id, user_id, role) VALUES (?1, ?2, ?3)",
            (project_id, user_id, role),
        )
        .await
        .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;
        Ok(())
    }

    pub async fn update_member_role(
        &self,
        project_id: i64,
        user_id: i64,
        role: &str,
    ) -> Result<(), AppError> {
        let conn = self
            .db
            .connect()
            .map_err(|e| AppError::Database(DbError::Connection(e.to_string())))?;
        conn.execute(
            "UPDATE project_members SET role = ?3
             WHERE project_id = ?1 AND user_id = ?2 AND role != 'owner'",
            (project_id, user_id, role),
        )
        .await
        .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;
        Ok(())
    }

    pub async fn remove_member(&self, project_id: i64, user_id: i64) -> Result<(), AppError> {
        let conn = self
            .db
            .connect()
            .map_err(|e| AppError::Database(DbError::Connection(e.to_string())))?;
        conn.execute(
            "DELETE FROM project_members WHERE project_id = ?1 AND user_id = ?2 AND role != 'owner'",
            (project_id, user_id),
        )
        .await
        .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, time::SystemTime};

    use super::*;
    use turso::Builder;

    #[tokio::test]
    async fn project_members_can_access_and_be_managed_without_removing_the_owner() {
        let suffix = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("system clock should be after the Unix epoch")
            .as_nanos();
        let db_path = std::env::temp_dir().join(format!("wemove-project-members-{suffix}.db"));
        let db_path_string = db_path.to_string_lossy();
        let db = Builder::new_local(db_path_string.as_ref())
            .build()
            .await
            .expect("test database should be created");
        let conn = db.connect().expect("test database should connect");
        conn.execute(
            "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL, email TEXT NOT NULL)",
            (),
        )
        .await
        .expect("users table should be created");
        conn.execute(
            "CREATE TABLE workspace_members (
                workspace_id INTEGER NOT NULL,
                user_id INTEGER NOT NULL,
                role TEXT NOT NULL,
                PRIMARY KEY (workspace_id, user_id)
            )",
            (),
        )
        .await
        .expect("workspace members table should be created");
        conn.execute(
            "CREATE TABLE projects (
                id INTEGER PRIMARY KEY,
                user_id INTEGER NOT NULL,
                workspace_id INTEGER NOT NULL,
                name TEXT NOT NULL
            )",
            (),
        )
        .await
        .expect("projects table should be created");
        conn.execute(
            "CREATE TABLE project_members (
                project_id INTEGER NOT NULL,
                user_id INTEGER NOT NULL,
                role TEXT NOT NULL CHECK (role IN ('owner', 'admin', 'member', 'viewer')),
                PRIMARY KEY (project_id, user_id)
            )",
            (),
        )
        .await
        .expect("project members table should be created");
        conn.execute(
            "INSERT INTO users (id, name, email) VALUES
                (1, 'Owner', 'owner@example.com'),
                (2, 'Participant', 'participant@example.com')",
            (),
        )
        .await
        .expect("users should be inserted");
        conn.execute(
            "INSERT INTO projects (id, user_id, workspace_id, name) VALUES (1, 1, 10, 'Roadmap')",
            (),
        )
        .await
        .expect("project should be inserted");
        conn.execute(
            "INSERT INTO project_members (project_id, user_id, role) VALUES (1, 1, 'owner')",
            (),
        )
        .await
        .expect("project owner should be inserted");

        let repository = ProjectRepository::new(db.clone());
        assert_eq!(
            repository
                .role_for_user(1, 1)
                .await
                .expect("owner role should load"),
            Some("owner".to_owned())
        );
        repository
            .ensure_workspace_member(1, 2)
            .await
            .expect("project participant should be added to the workspace");
        repository
            .add_member(1, 2, "member")
            .await
            .expect("project participant should be added");
        assert!(
            repository
                .find_for_user(2, 1)
                .await
                .expect("participant project lookup should work")
                .is_some()
        );
        assert_eq!(
            repository
                .list_members(1)
                .await
                .expect("project members should load"),
            [
                ProjectMember {
                    user_id: 1,
                    name: "Owner".to_owned(),
                    email: "owner@example.com".to_owned(),
                    role: "owner".to_owned()
                },
                ProjectMember {
                    user_id: 2,
                    name: "Participant".to_owned(),
                    email: "participant@example.com".to_owned(),
                    role: "member".to_owned()
                },
            ]
        );
        repository
            .update_member_role(1, 2, "viewer")
            .await
            .expect("participant role should be updated");
        repository
            .remove_member(1, 1)
            .await
            .expect("owner removal should be safely ignored");
        assert_eq!(
            repository
                .role_for_user(1, 1)
                .await
                .expect("owner role should remain"),
            Some("owner".to_owned())
        );
        repository
            .remove_member(1, 2)
            .await
            .expect("participant should be removable");
        assert_eq!(
            repository
                .role_for_user(1, 2)
                .await
                .expect("removed role should load"),
            None
        );

        drop(repository);
        drop(conn);
        drop(db);
        fs::remove_file(db_path).expect("test database should be removed");
    }
}
