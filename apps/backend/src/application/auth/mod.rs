pub mod login;
pub mod logout;
pub mod refresh;
pub mod register;

use uuid::Uuid;

use crate::domain::user::entity::User;

/// Produced by register/login (Task 6) after a session is created and an
/// access token is signed.
///
/// Invariant: `sid == family_id` — the access token's `sid` claim IS the
/// refresh-token family id.
pub struct AuthSession {
    pub access_token: String,
    pub family_id: Uuid,
    pub raw_secret: String,
    pub sid: Uuid,
}

/// Result of register/login: the authenticated [`User`] plus the freshly-issued
/// [`AuthSession`]. The handler builds both the cookie jar (from the session)
/// and the response body (from the user) without a second database read.
pub struct AuthOutcome {
    pub user: User,
    pub session: AuthSession,
}
