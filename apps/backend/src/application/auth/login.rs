use std::sync::Arc;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use uuid::Uuid;

use crate::{
    application::{
        auth::{AuthOutcome, AuthSession},
        errors::AppError,
    },
    domain::user::{entity::User, value_objects::Email},
    ports::{
        auth::AuthPort,
        repositories::{RepositoryError, UserRepository},
        session::SessionStore,
    },
};

pub struct LoginUseCase {
    pub user_repo: Arc<dyn UserRepository>,
    pub auth: Arc<dyn AuthPort>,
    pub sessions: Arc<dyn SessionStore>,
    pub access_ttl_secs: u64,
}

impl LoginUseCase {
    /// Verify credentials, create a refresh-token session family, and mint an
    /// access token whose `sid` claim equals the family id. Returns the
    /// authenticated user alongside the session so the handler needs no second
    /// database read for the response body.
    pub async fn execute(&self, email: String, password: String) -> Result<AuthOutcome, AppError> {
        let email_vo = Email::new(email).map_err(AppError::from)?;

        let user = self
            .user_repo
            .find_by_email(email_vo.as_str())
            .await
            .map_err(|e| match e {
                RepositoryError::NotFound => AppError::Unauthorized,
                other => AppError::from(other),
            })?;

        let parsed_hash = PasswordHash::new(user.password_hash.as_str())
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::Unauthorized)?;

        issue_session(self.sessions.as_ref(), self.auth.as_ref(), user).await
    }
}

/// Shared session-minting step for register/login: create a family, sign an
/// access token with `sid == family_id`, and bundle the result with the user.
///
/// Kept as a free function so register and login never duplicate the
/// create-session + sign-access sequence. Takes ownership of the [`User`] (the
/// caller already has it) and returns it in the [`AuthOutcome`].
pub(crate) async fn issue_session(
    sessions: &dyn SessionStore,
    auth: &dyn AuthPort,
    user: User,
) -> Result<AuthOutcome, AppError> {
    let session = sessions
        .create(user.id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let sid = session.family_id; // invariant: sid == family_id
    let jti = Uuid::new_v4();
    let access_token = auth
        .sign_access(user.id, sid, jti)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(AuthOutcome {
        user,
        session: AuthSession {
            access_token,
            family_id: session.family_id,
            raw_secret: session.raw_secret,
            sid,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use argon2::{
        password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
        Argon2 as Argon2Hash,
    };
    use async_trait::async_trait;
    use uuid::Uuid;

    use crate::domain::user::{
        entity::User,
        value_objects::{Email, PasswordHash as DomainPasswordHash},
    };
    use crate::ports::session::{NewSession, RotateOutcome, SessionError, SessionResult};

    struct MockUserRepo {
        email: String,
        password_hash: String,
    }

    #[async_trait::async_trait]
    impl crate::ports::repositories::UserRepository for MockUserRepo {
        async fn find_by_email(
            &self,
            email: &str,
        ) -> crate::ports::repositories::RepositoryResult<User> {
            if email == self.email {
                Ok(User {
                    id: Uuid::new_v4(),
                    email: Email::new(email.to_string()).unwrap(),
                    password_hash: DomainPasswordHash::from_hash(self.password_hash.clone()),
                    name: "Alice".into(),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                })
            } else {
                Err(crate::ports::repositories::RepositoryError::NotFound)
            }
        }

        async fn find_by_id(
            &self,
            _id: Uuid,
        ) -> crate::ports::repositories::RepositoryResult<User> {
            Err(crate::ports::repositories::RepositoryError::NotFound)
        }

        async fn insert(
            &self,
            _email: &str,
            _hash: &str,
            _name: &str,
        ) -> crate::ports::repositories::RepositoryResult<User> {
            Err(crate::ports::repositories::RepositoryError::NotFound)
        }
    }

    struct MockAuth;
    impl crate::ports::auth::AuthPort for MockAuth {
        fn sign_access(
            &self,
            _user_id: Uuid,
            _sid: Uuid,
            _jti: Uuid,
        ) -> Result<String, crate::ports::auth::AuthError> {
            Ok("mock.access.token".into())
        }
        fn verify_access(
            &self,
            _token: &str,
        ) -> Result<crate::ports::auth::AccessClaims, crate::ports::auth::AuthError> {
            Ok(crate::ports::auth::AccessClaims {
                user_id: Uuid::new_v4(),
                sid: Uuid::new_v4(),
                jti: Uuid::new_v4(),
            })
        }
    }

    struct MockSessions;
    #[async_trait]
    impl SessionStore for MockSessions {
        async fn create(&self, _user_id: Uuid) -> SessionResult<NewSession> {
            Ok(NewSession {
                family_id: Uuid::new_v4(),
                raw_secret: "raw-secret".into(),
            })
        }
        async fn rotate(&self, _f: Uuid, _s: &str) -> SessionResult<RotateOutcome> {
            Err(SessionError::Unavailable("unused".into()))
        }
        async fn revoke(&self, _f: Uuid) -> SessionResult<()> {
            Ok(())
        }
        async fn revoke_session(&self, _sid: Uuid, _ttl: u64) -> SessionResult<()> {
            Ok(())
        }
        async fn is_session_revoked(&self, _sid: Uuid) -> SessionResult<bool> {
            Ok(false)
        }
    }

    fn make_hash(password: &str) -> String {
        let salt = SaltString::generate(&mut OsRng);
        Argon2Hash::default()
            .hash_password(password.as_bytes(), &salt)
            .unwrap()
            .to_string()
    }

    fn make_uc(email: &str, password_hash: String) -> LoginUseCase {
        LoginUseCase {
            user_repo: Arc::new(MockUserRepo {
                email: email.into(),
                password_hash,
            }),
            auth: Arc::new(MockAuth),
            sessions: Arc::new(MockSessions),
            access_ttl_secs: 900,
        }
    }

    #[tokio::test]
    async fn login_returns_session_on_correct_credentials() {
        let uc = make_uc("alice@example.com", make_hash("password123"));
        let result = uc
            .execute("alice@example.com".into(), "password123".into())
            .await;
        let outcome = result.expect("expected Ok");
        assert_eq!(outcome.session.access_token, "mock.access.token");
        assert_eq!(outcome.session.sid, outcome.session.family_id);
        assert_eq!(outcome.session.raw_secret, "raw-secret");
        // Fix 4: the use case returns the user, so the handler needs no 2nd read.
        assert_eq!(outcome.user.email.as_str(), "alice@example.com");
    }

    #[tokio::test]
    async fn login_rejects_wrong_password() {
        let uc = make_uc("alice@example.com", make_hash("password123"));
        let result = uc
            .execute("alice@example.com".into(), "wrongpassword".into())
            .await;
        assert!(matches!(result, Err(AppError::Unauthorized)));
    }

    #[tokio::test]
    async fn login_rejects_unknown_email() {
        let uc = make_uc("alice@example.com", make_hash("password123"));
        let result = uc
            .execute("nobody@example.com".into(), "password123".into())
            .await;
        assert!(matches!(result, Err(AppError::Unauthorized)));
    }
}
