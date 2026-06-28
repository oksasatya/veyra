use crate::domain::errors::DomainError;

/// A validated, lowercase-normalised email address.
///
/// # Errors
/// Returns [`DomainError::InvalidEmail`] when the value is empty, has no `@`,
/// has an empty local part, or has a domain with no `.`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Email(String);

impl Email {
    pub fn new(raw: String) -> Result<Self, DomainError> {
        let normalized = raw.trim().to_lowercase();

        let Some(at_pos) = normalized.find('@') else {
            return Err(DomainError::InvalidEmail(raw));
        };

        let local = &normalized[..at_pos];
        let domain = &normalized[at_pos + 1..];

        if local.is_empty() || domain.is_empty() || !domain.contains('.') {
            return Err(DomainError::InvalidEmail(raw));
        }

        Ok(Self(normalized))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<Email> for String {
    fn from(e: Email) -> Self {
        e.0
    }
}

/// A pre-validated argon2 password hash. No validation is performed here —
/// the hash must be produced by the auth adapter before calling `from_hash`.
#[derive(Debug, Clone)]
pub struct PasswordHash(String);

impl PasswordHash {
    pub fn from_hash(hash: String) -> Self {
        Self(hash)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<PasswordHash> for String {
    fn from(p: PasswordHash) -> Self {
        p.0
    }
}

/// The user's preferred language for server-generated content. Static UI copy is
/// localized by the client; this drives only what the backend renders itself
/// (notifications, emails). Defaults to English.
///
/// The wire/storage representation is the lowercase code from [`Language::as_str`]
/// (`"en"` / `"id"`); the DB enforces the same set via a CHECK constraint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    En,
    Id,
}

impl Language {
    /// Parse a language code (case-insensitive, trimmed).
    ///
    /// # Errors
    /// Returns [`DomainError::InvalidLanguage`] for any value other than
    /// `"en"` or `"id"`.
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        match raw.trim().to_lowercase().as_str() {
            "en" => Ok(Language::En),
            "id" => Ok(Language::Id),
            _ => Err(DomainError::InvalidLanguage),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Language::En => "en",
            Language::Id => "id",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_email_accepted() {
        let e = Email::new("alice@example.com".to_string());
        assert!(e.is_ok());
        assert_eq!(e.unwrap().as_str(), "alice@example.com");
    }

    #[test]
    fn email_normalised_to_lowercase() {
        let e = Email::new("Alice@Example.COM".to_string()).unwrap();
        assert_eq!(e.as_str(), "alice@example.com");
    }

    #[test]
    fn email_without_at_rejected() {
        assert!(Email::new("notanemail".to_string()).is_err());
    }

    #[test]
    fn empty_email_rejected() {
        assert!(Email::new(String::new()).is_err());
    }

    #[test]
    fn email_no_domain_dot_rejected() {
        assert!(Email::new("a@nodot".to_string()).is_err());
    }

    #[test]
    fn language_parses_known_codes_case_insensitively() {
        assert_eq!(Language::parse("en").unwrap(), Language::En);
        assert_eq!(Language::parse("ID").unwrap(), Language::Id);
        assert_eq!(Language::parse(" id ").unwrap(), Language::Id);
    }

    #[test]
    fn language_rejects_unknown_code() {
        assert!(matches!(
            Language::parse("fr"),
            Err(DomainError::InvalidLanguage)
        ));
    }

    #[test]
    fn language_default_is_english() {
        assert_eq!(Language::default(), Language::En);
        assert_eq!(Language::En.as_str(), "en");
        assert_eq!(Language::Id.as_str(), "id");
    }
}
