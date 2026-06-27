use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ports::auth::{AccessClaims, AuthError, AuthPort};

// ── Serde claim structs live in the adapter — never in ports/ ────────────────

/// Claims for the legacy 7-day tokens (removed in Task 6).
#[derive(Debug, Serialize, Deserialize)]
struct LegacyClaims {
    sub: String,
    exp: i64,
}

/// Claims for the new short-lived access tokens.
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    sid: String,
    jti: String,
    iat: i64,
    exp: i64,
}

// ── JwtAuth ──────────────────────────────────────────────────────────────────

/// JWT implementation of [`AuthPort`].
///
/// Signs HS256 tokens. `access_ttl_secs` governs the lifetime of tokens
/// created via [`sign_access`]. The legacy [`sign_token`] keeps its fixed 7-day
/// TTL until it is removed in Task 6.
#[derive(Clone)]
pub struct JwtAuth {
    secret: String,
    access_ttl_secs: u64,
}

impl JwtAuth {
    pub fn new(secret: String, access_ttl_secs: u64) -> Self {
        Self {
            secret,
            access_ttl_secs,
        }
    }
}

impl AuthPort for JwtAuth {
    // ── Legacy (removed in Task 6) ────────────────────────────────────────────
    fn sign_token(&self, user_id: Uuid) -> Result<String, AuthError> {
        let exp = (Utc::now() + Duration::days(7)).timestamp();
        let claims = LegacyClaims {
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
        let token_data = decode::<LegacyClaims>(
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

    // ── New access-token methods ──────────────────────────────────────────────
    fn sign_access(&self, user_id: Uuid, sid: Uuid, jti: Uuid) -> Result<String, AuthError> {
        let now = Utc::now();
        let iat = now.timestamp();
        #[allow(clippy::cast_possible_wrap)]
        let exp = (now + Duration::seconds(self.access_ttl_secs as i64)).timestamp();
        let claims = Claims {
            sub: user_id.to_string(),
            sid: sid.to_string(),
            jti: jti.to_string(),
            iat,
            exp,
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| AuthError::SigningFailed(e.to_string()))
    }

    fn verify_access(&self, token: &str) -> Result<AccessClaims, AuthError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.leeway = 0; // no grace window — expire exactly at `exp`
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )
        .map_err(|_| AuthError::InvalidToken)?;

        let c = token_data.claims;
        let user_id = c.sub.parse::<Uuid>().map_err(|_| AuthError::InvalidToken)?;
        let sid = c.sid.parse::<Uuid>().map_err(|_| AuthError::InvalidToken)?;
        let jti = c.jti.parse::<Uuid>().map_err(|_| AuthError::InvalidToken)?;

        Ok(AccessClaims { user_id, sid, jti })
    }
}

// ── Unit tests (TDD: written first, then the impl above makes them pass) ─────

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "test-secret-32chars-at-minimum!!";

    #[test]
    fn sign_and_verify_preserves_claims() {
        let jwt = JwtAuth::new(SECRET.into(), 900);
        let user_id = Uuid::new_v4();
        let sid = Uuid::new_v4();
        let jti = Uuid::new_v4();
        let token = jwt.sign_access(user_id, sid, jti).expect("sign_access ok");
        let claims = jwt.verify_access(&token).expect("verify_access ok");
        assert_eq!(claims.user_id, user_id);
        assert_eq!(claims.sid, sid);
        assert_eq!(claims.jti, jti);
    }

    #[test]
    fn verify_rejects_wrong_secret() {
        let jwt_a = JwtAuth::new("secret-a-32chars-at-minimum--!!".into(), 900);
        let jwt_b = JwtAuth::new("secret-b-32chars-at-minimum--!!".into(), 900);
        let token = jwt_a
            .sign_access(Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4())
            .expect("sign ok");
        let result = jwt_b.verify_access(&token);
        assert!(matches!(result, Err(AuthError::InvalidToken)));
    }

    #[test]
    fn verify_rejects_expired() {
        // Build a token whose exp is 10 seconds in the past.
        // verify_access sets leeway = 0, so it must be rejected.
        let past_exp = Utc::now().timestamp() - 10;
        let claims = Claims {
            sub: Uuid::new_v4().to_string(),
            sid: Uuid::new_v4().to_string(),
            jti: Uuid::new_v4().to_string(),
            iat: past_exp - 900,
            exp: past_exp,
        };
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(SECRET.as_bytes()),
        )
        .expect("encode ok");
        let jwt = JwtAuth::new(SECRET.into(), 900);
        let result = jwt.verify_access(&token);
        assert!(matches!(result, Err(AuthError::InvalidToken)));
    }

    // ── Legacy tests preserved from postgres/jwt_auth.rs ─────────────────────

    #[test]
    fn sign_and_verify_roundtrip() {
        let jwt = JwtAuth::new(SECRET.into(), 900);
        let user_id = Uuid::new_v4();
        let token = jwt.sign_token(user_id).unwrap();
        let parsed = jwt.verify_token(&token).unwrap();
        assert_eq!(parsed, user_id);
    }

    #[test]
    fn verify_invalid_token_returns_error() {
        let jwt = JwtAuth::new(SECRET.into(), 900);
        let result = jwt.verify_token("not.a.valid.jwt");
        assert!(matches!(result, Err(AuthError::InvalidToken)));
    }

    #[test]
    fn verify_token_signed_with_different_secret_fails() {
        let jwt_a = JwtAuth::new("secret-a-32chars-at-minimum--!!".into(), 900);
        let jwt_b = JwtAuth::new("secret-b-32chars-at-minimum--!!".into(), 900);
        let user_id = Uuid::new_v4();
        let token = jwt_a.sign_token(user_id).unwrap();
        let result = jwt_b.verify_token(&token);
        assert!(matches!(result, Err(AuthError::InvalidToken)));
    }
}
