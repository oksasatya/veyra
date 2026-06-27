use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ports::auth::{AuthError, AuthPort};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: i64,
}

/// JWT implementation of [`AuthPort`]. Signs tokens with HS256 and a secret
/// sourced from the application configuration — never from source code.
#[derive(Clone)]
pub struct JwtAuth {
    secret: String,
}

impl JwtAuth {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }
}

impl AuthPort for JwtAuth {
    fn sign_token(&self, user_id: Uuid) -> Result<String, AuthError> {
        let exp = (Utc::now() + Duration::days(7)).timestamp();
        let claims = Claims {
            sub: user_id.to_string(),
            exp,
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| AuthError::SigningFailed(e.to_string()))
    }

    fn verify_token(&self, token: &str) -> Result<Uuid, AuthError> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|_| AuthError::InvalidToken)?;

        token_data
            .claims
            .sub
            .parse::<Uuid>()
            .map_err(|_| AuthError::InvalidToken)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_and_verify_roundtrip() {
        let jwt = JwtAuth::new("test-secret-32chars-at-minimum!!".into());
        let user_id = Uuid::new_v4();
        let token = jwt.sign_token(user_id).unwrap();
        let parsed = jwt.verify_token(&token).unwrap();
        assert_eq!(parsed, user_id);
    }

    #[test]
    fn verify_invalid_token_returns_error() {
        let jwt = JwtAuth::new("test-secret-32chars-at-minimum!!".into());
        let result = jwt.verify_token("not.a.valid.jwt");
        assert!(matches!(result, Err(AuthError::InvalidToken)));
    }

    #[test]
    fn verify_token_signed_with_different_secret_fails() {
        let jwt_a = JwtAuth::new("secret-a-32chars-at-minimum--!!".into());
        let jwt_b = JwtAuth::new("secret-b-32chars-at-minimum--!!".into());
        let user_id = Uuid::new_v4();
        let token = jwt_a.sign_token(user_id).unwrap();
        let result = jwt_b.verify_token(&token);
        assert!(matches!(result, Err(AuthError::InvalidToken)));
    }
}
