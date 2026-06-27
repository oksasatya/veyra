use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("invalid or expired token")]
    InvalidToken,
    #[error("token generation failed: {0}")]
    SigningFailed(String),
}

/// Claims returned by [`AuthPort::verify_access`].
///
/// `sid` is the session family id (== the refresh family_id).
/// `jti` is a per-token unique id used for future fine-grained revocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessClaims {
    pub user_id: Uuid,
    pub sid: Uuid,
    pub jti: Uuid,
}

pub trait AuthPort: Send + Sync {
    /// Sign a short-lived access token carrying `{ sub, sid, jti, iat, exp }`.
    fn sign_access(&self, user_id: Uuid, sid: Uuid, jti: Uuid) -> Result<String, AuthError>;
    /// Verify an access token and return its claims.
    fn verify_access(&self, token: &str) -> Result<AccessClaims, AuthError>;
}
