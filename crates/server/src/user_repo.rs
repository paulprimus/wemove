use bcrypt::{hash, verify, DEFAULT_COST};
use common::error::{AppError, DbError};
use turso::Database;

#[derive(Debug, Clone)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub name: String,
    pub password_hash: String,
}

#[derive(Debug)]
pub struct CreateUser {
    pub email: String,
    pub name: String,
    pub password: String,
}

pub struct UserRepository {
    db: Database,
}

impl UserRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let conn = self
            .db
            .connect()
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let mut stmt = conn
            .prepare("SELECT id, email, name, password_hash FROM users WHERE email = ?1")
            .await
            .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;

        let mut rows = stmt
            .query([email])
            .await
            .map_err(|e| AppError::Database(DbError::Query(e.to_string())))?;

        match rows.next().await {
            Ok(Some(row)) => {
                let id = row
                    .get_value(0)
                    .map_err(|e| AppError::Internal(format!("Failed to get id: {}", e)))?;
                let email = row.get_value(1).map_err(|e| {
                    AppError::Internal(format!("Failed to get email: {}", e))
                })?;
                let name = row.get_value(2).map_err(|e| {
                    AppError::Internal(format!("Failed to get name: {}", e))
                })?;
                let password_hash = row.get_value(3).map_err(|e| {
                    AppError::Internal(format!("Failed to get password_hash: {}", e))
                })?;

                let user = User {
                    id: match id {
                        turso::Value::Integer(i) => i,
                        _ => return Err(AppError::Internal("Invalid id type".to_string())),
                    },
                    email: match email {
                        turso::Value::Text(s) => s,
                        _ => return Err(AppError::Internal("Invalid email type".to_string())),
                    },
                    name: match name {
                        turso::Value::Text(s) => s,
                        _ => return Err(AppError::Internal("Invalid name type".to_string())),
                    },
                    password_hash: match password_hash {
                        turso::Value::Text(s) => s,
                        _ => {
                            return Err(AppError::Internal(
                                "Invalid password_hash type".to_string(),
                            ))
                        }
                    },
                };

                Ok(Some(user))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(AppError::Database(DbError::Query(e.to_string()))),
        }
    }

    pub async fn create(&self, user: CreateUser) -> Result<i64, AppError> {
        let password_hash = hash(&user.password, DEFAULT_COST)
            .map_err(|e| AppError::Database(DbError::PasswordHash(e.to_string())))?;

        let conn = self
            .db
            .connect()
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let result = conn
            .execute(
                "INSERT INTO users (email, name, password_hash) VALUES (?1, ?2, ?3)",
                (user.email.as_str(), user.name.as_str(), password_hash.as_str()),
            )
            .await
            .map_err(|e| {
                if e.to_string().contains("UNIQUE constraint") {
                    AppError::Conflict("Email already exists".to_string())
                } else {
                    AppError::Database(DbError::Query(e.to_string()))
                }
            })?;

        Ok(result as i64)
    }

    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, AppError> {
        verify(password, hash)
            .map_err(|e| AppError::Database(DbError::PasswordHash(e.to_string())))
    }
}