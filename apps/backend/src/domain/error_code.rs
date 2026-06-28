//! Stable, machine-readable error codes.
//!
//! These codes are the i18n contract between the API and its clients. The
//! backend returns a stable `ErrorCode` (e.g. `INVALID_PLATE_NUMBER`); the
//! client (the Flutter app) maps that code to a localized message in the user's
//! language. The human-readable `message` carried alongside a code is English
//! developer prose for logs/debugging — clients localize from the CODE, never by
//! displaying `message`.
//!
//! This type lives in `domain` and is intentionally serde-free (the domain layer
//! imports no framework). The HTTP adapter serializes it via [`ErrorCode::as_str`].

/// A stable, machine-readable error identifier. Rendered on the wire as the
/// SCREAMING_SNAKE_CASE string from [`ErrorCode::as_str`].
///
/// Granular where the client needs distinct localized copy (validation), generic
/// where the HTTP route already conveys the resource (`NotFound`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    // ── Auth / user ──────────────────────────────────────────────────────────
    InvalidEmail,
    PasswordTooShort,
    EmailAlreadyExists,
    InvalidLanguage,
    Unauthorized,

    // ── Vehicle ──────────────────────────────────────────────────────────────
    InvalidPlateNumber,
    OdometerDecreased,
    InvalidFuelType,

    // ── Reminder ─────────────────────────────────────────────────────────────
    InvalidReminderType,
    MissingDueDate,
    MissingDueOdometer,

    // ── Expense / document ────────────────────────────────────────────────────
    InvalidCategory,
    InvalidDocType,

    // ── Generic ───────────────────────────────────────────────────────────────
    NotFound,
    Conflict,
    Validation,
    Forbidden,
    ServiceUnavailable,
    Internal,
}

impl ErrorCode {
    /// The stable wire representation. This string is part of the public API
    /// contract — clients key their localized messages off it, so changing one
    /// is a breaking change.
    pub fn as_str(self) -> &'static str {
        match self {
            ErrorCode::InvalidEmail => "INVALID_EMAIL",
            ErrorCode::PasswordTooShort => "PASSWORD_TOO_SHORT",
            ErrorCode::EmailAlreadyExists => "EMAIL_ALREADY_EXISTS",
            ErrorCode::InvalidLanguage => "INVALID_LANGUAGE",
            ErrorCode::Unauthorized => "UNAUTHORIZED",
            ErrorCode::InvalidPlateNumber => "INVALID_PLATE_NUMBER",
            ErrorCode::OdometerDecreased => "ODOMETER_DECREASED",
            ErrorCode::InvalidFuelType => "INVALID_FUEL_TYPE",
            ErrorCode::InvalidReminderType => "INVALID_REMINDER_TYPE",
            ErrorCode::MissingDueDate => "MISSING_DUE_DATE",
            ErrorCode::MissingDueOdometer => "MISSING_DUE_ODOMETER",
            ErrorCode::InvalidCategory => "INVALID_CATEGORY",
            ErrorCode::InvalidDocType => "INVALID_DOC_TYPE",
            ErrorCode::NotFound => "NOT_FOUND",
            ErrorCode::Conflict => "CONFLICT",
            ErrorCode::Validation => "VALIDATION",
            ErrorCode::Forbidden => "FORBIDDEN",
            ErrorCode::ServiceUnavailable => "SERVICE_UNAVAILABLE",
            ErrorCode::Internal => "INTERNAL",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_str_is_screaming_snake_case() {
        let cases = [
            (ErrorCode::InvalidEmail, "INVALID_EMAIL"),
            (ErrorCode::PasswordTooShort, "PASSWORD_TOO_SHORT"),
            (ErrorCode::EmailAlreadyExists, "EMAIL_ALREADY_EXISTS"),
            (ErrorCode::InvalidLanguage, "INVALID_LANGUAGE"),
            (ErrorCode::Unauthorized, "UNAUTHORIZED"),
            (ErrorCode::InvalidPlateNumber, "INVALID_PLATE_NUMBER"),
            (ErrorCode::OdometerDecreased, "ODOMETER_DECREASED"),
            (ErrorCode::InvalidFuelType, "INVALID_FUEL_TYPE"),
            (ErrorCode::InvalidReminderType, "INVALID_REMINDER_TYPE"),
            (ErrorCode::MissingDueDate, "MISSING_DUE_DATE"),
            (ErrorCode::MissingDueOdometer, "MISSING_DUE_ODOMETER"),
            (ErrorCode::InvalidCategory, "INVALID_CATEGORY"),
            (ErrorCode::InvalidDocType, "INVALID_DOC_TYPE"),
            (ErrorCode::NotFound, "NOT_FOUND"),
            (ErrorCode::Conflict, "CONFLICT"),
            (ErrorCode::Validation, "VALIDATION"),
            (ErrorCode::Forbidden, "FORBIDDEN"),
            (ErrorCode::ServiceUnavailable, "SERVICE_UNAVAILABLE"),
            (ErrorCode::Internal, "INTERNAL"),
        ];
        for (code, expected) in cases {
            assert_eq!(code.as_str(), expected);
        }
    }
}
