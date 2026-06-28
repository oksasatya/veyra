use thiserror::Error;

use crate::{
    domain::{error_code::ErrorCode, errors::DomainError},
    ports::repositories::RepositoryError,
};

/// Application-facing error. Each variant carries a stable [`ErrorCode`] (the i18n
/// contract — clients localize from the code) plus an English developer message for
/// logs. The HTTP adapter ([`crate::adapters::inbound::http::errors`]) is the single
/// place that maps a variant to an HTTP status; this layer stays framework-free.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("not found")]
    NotFound,
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("service unavailable")]
    Unavailable,
    #[error("{message}")]
    Conflict { code: ErrorCode, message: String },
    #[error("{message}")]
    Validation { code: ErrorCode, message: String },
    #[error("internal: {0}")]
    Internal(String),
}

impl AppError {
    /// Construct a 422 validation error with an explicit code.
    pub fn validation(code: ErrorCode, message: impl Into<String>) -> Self {
        AppError::Validation {
            code,
            message: message.into(),
        }
    }

    /// Construct a 409 conflict error with an explicit code.
    pub fn conflict(code: ErrorCode, message: impl Into<String>) -> Self {
        AppError::Conflict {
            code,
            message: message.into(),
        }
    }

    /// The stable wire code for this error. The HTTP adapter serializes this; the
    /// status code is derived separately at the adapter (it owns axum types).
    pub fn code(&self) -> ErrorCode {
        match self {
            AppError::NotFound => ErrorCode::NotFound,
            AppError::Unauthorized => ErrorCode::Unauthorized,
            AppError::Forbidden => ErrorCode::Forbidden,
            AppError::Unavailable => ErrorCode::ServiceUnavailable,
            AppError::Conflict { code, .. } | AppError::Validation { code, .. } => *code,
            AppError::Internal(_) => ErrorCode::Internal,
        }
    }
}

impl From<DomainError> for AppError {
    fn from(e: DomainError) -> Self {
        AppError::Validation {
            code: e.code(),
            message: e.to_string(),
        }
    }
}

impl From<RepositoryError> for AppError {
    fn from(e: RepositoryError) -> Self {
        match e {
            RepositoryError::NotFound => AppError::NotFound,
            RepositoryError::Conflict(msg) => AppError::Conflict {
                code: ErrorCode::Conflict,
                message: msg,
            },
            RepositoryError::Database(msg) => AppError::Internal(msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_maps_each_variant() {
        assert_eq!(AppError::NotFound.code(), ErrorCode::NotFound);
        assert_eq!(AppError::Unauthorized.code(), ErrorCode::Unauthorized);
        assert_eq!(AppError::Internal("x".into()).code(), ErrorCode::Internal);
        assert_eq!(
            AppError::validation(ErrorCode::InvalidFuelType, "x").code(),
            ErrorCode::InvalidFuelType
        );
        assert_eq!(
            AppError::conflict(ErrorCode::EmailAlreadyExists, "x").code(),
            ErrorCode::EmailAlreadyExists
        );
    }

    #[test]
    fn from_domain_error_carries_code() {
        let err: AppError = DomainError::InvalidPlateNumber("ABC".into()).into();
        assert_eq!(err.code(), ErrorCode::InvalidPlateNumber);
    }

    #[test]
    fn from_repository_conflict_is_generic_conflict_code() {
        let err: AppError = RepositoryError::Conflict("dup".into()).into();
        assert_eq!(err.code(), ErrorCode::Conflict);
    }
}
