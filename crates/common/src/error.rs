use thiserror::Error;
use utoipa::ToSchema;

/// Framework-agnostisches Fehler-Enum. Die Übersetzung in eine konkrete
/// HTTP-Response (z. B. via `axum::response::IntoResponse`) obliegt dem
/// jeweiligen Web-Layer (siehe `server`-Crate), damit `common` nicht an
/// ein bestimmtes Web-Framework gebunden ist.
///
/// `Internal` nimmt bewusst nur eine `String`-Message auf (statt z. B.
/// `anyhow::Error`), damit `common` keine Abhängigkeit zu `anyhow` benötigt.
/// Aufrufer, die mit `anyhow::Error` arbeiten, wandeln diesen selbst um
/// (z. B. via `.map_err(|e: anyhow::Error| AppError::Internal(e.to_string()))`).
#[derive(Error, Debug, ToSchema)]
pub enum AppError {
    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Not found: {0}")]
    NotFound(String),
}