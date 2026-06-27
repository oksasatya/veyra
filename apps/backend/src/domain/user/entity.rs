use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::value_objects::{Email, PasswordHash};

/// The User domain entity. All fields are publicly readable; mutation is
/// intentionally absent — changes go through use-case methods when needed.
#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub email: Email,
    pub password_hash: PasswordHash,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
