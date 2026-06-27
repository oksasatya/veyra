use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("invalid email: {0}")]
    InvalidEmail(String),

    #[error("password too short — minimum 8 characters")]
    PasswordTooShort,

    #[error("invalid plate number: {0}")]
    InvalidPlateNumber(String),

    #[error("odometer value cannot decrease: current={current}, proposed={proposed}")]
    OdometerDecreased { current: u32, proposed: u32 },

    #[error("missing due_date for date-type reminder")]
    MissingDueDate,

    #[error("missing due_odometer for odometer-type reminder")]
    MissingDueOdometer,
}
