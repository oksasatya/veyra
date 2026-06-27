use thiserror::Error;

use crate::{domain::errors::DomainError, ports::repositories::RepositoryError};

#[derive(Debug, Error)]
pub enum AppError {
    #[error("not found")]
    NotFound,
    #[error("unauthorized")]
    Unauthorized,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("validation: {0}")]
    Validation(String),
    #[error("internal: {0}")]
    Internal(String),
}

impl From<DomainError> for AppError {
    fn from(e: DomainError) -> Self {
        AppError::Validation(e.to_string())
    }
}

impl From<RepositoryError> for AppError {
    fn from(e: RepositoryError) -> Self {
        match e {
            RepositoryError::NotFound => AppError::NotFound,
            RepositoryError::Conflict(msg) => AppError::Conflict(msg),
            RepositoryError::Database(msg) => AppError::Internal(msg),
        }
    }
}
