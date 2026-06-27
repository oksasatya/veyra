use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("invalid or expired token")]
    InvalidToken,
    #[error("token generation failed: {0}")]
    SigningFailed(String),
}

pub trait AuthPort: Send + Sync {
    fn sign_token(&self, user_id: Uuid) -> Result<String, AuthError>;
    fn verify_token(&self, token: &str) -> Result<Uuid, AuthError>;
}
