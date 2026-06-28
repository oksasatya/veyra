use thiserror::Error;

use crate::domain::error_code::ErrorCode;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("invalid email: {0}")]
    InvalidEmail(String),

    #[error("password too short — minimum 8 characters")]
    PasswordTooShort,

    #[error("unsupported language — allowed values are 'en' and 'id'")]
    InvalidLanguage,

    #[error("invalid plate number: {0}")]
    InvalidPlateNumber(String),

    #[error("odometer value cannot decrease: current={current}, proposed={proposed}")]
    OdometerDecreased { current: u32, proposed: u32 },

    #[error("missing due_date for date-type reminder")]
    MissingDueDate,

    #[error("missing due_odometer for odometer-type reminder")]
    MissingDueOdometer,
}

impl DomainError {
    /// The stable [`ErrorCode`] for this error. This is the choke point that maps
    /// each domain failure to its wire code — adding a variant makes this match
    /// non-exhaustive (a compile error), so a new error can never ship without a code.
    pub fn code(&self) -> ErrorCode {
        match self {
            DomainError::InvalidEmail(_) => ErrorCode::InvalidEmail,
            DomainError::PasswordTooShort => ErrorCode::PasswordTooShort,
            DomainError::InvalidLanguage => ErrorCode::InvalidLanguage,
            DomainError::InvalidPlateNumber(_) => ErrorCode::InvalidPlateNumber,
            DomainError::OdometerDecreased { .. } => ErrorCode::OdometerDecreased,
            DomainError::MissingDueDate => ErrorCode::MissingDueDate,
            DomainError::MissingDueOdometer => ErrorCode::MissingDueOdometer,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_maps_each_variant() {
        let cases: [(DomainError, ErrorCode); 7] = [
            (
                DomainError::InvalidEmail("x".into()),
                ErrorCode::InvalidEmail,
            ),
            (DomainError::PasswordTooShort, ErrorCode::PasswordTooShort),
            (DomainError::InvalidLanguage, ErrorCode::InvalidLanguage),
            (
                DomainError::InvalidPlateNumber("x".into()),
                ErrorCode::InvalidPlateNumber,
            ),
            (
                DomainError::OdometerDecreased {
                    current: 10,
                    proposed: 5,
                },
                ErrorCode::OdometerDecreased,
            ),
            (DomainError::MissingDueDate, ErrorCode::MissingDueDate),
            (
                DomainError::MissingDueOdometer,
                ErrorCode::MissingDueOdometer,
            ),
        ];
        for (err, expected) in cases {
            assert_eq!(err.code(), expected);
        }
    }
}
