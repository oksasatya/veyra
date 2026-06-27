use std::sync::Arc;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};

use crate::{
    application::{auth::login::issue_session, auth::AuthOutcome, errors::AppError},
    domain::{errors::DomainError, user::value_objects::Email},
    ports::{auth::AuthPort, repositories::UserRepository, session::SessionStore},
};

pub struct RegisterUseCase {
    pub user_repo: Arc<dyn UserRepository>,
    pub auth: Arc<dyn AuthPort>,
    pub sessions: Arc<dyn SessionStore>,
    pub access_ttl_secs: u64,
}

impl RegisterUseCase {
    /// Create a user, then create a refresh-token session family and mint an
    /// access token whose `sid` claim equals the family id.
    pub async fn execute(
        &self,
        email: String,
        password: String,
        name: String,
    ) -> Result<AuthOutcome, AppError> {
        let email_vo = Email::new(email).map_err(AppError::from)?;

        if password.len() < 8 {
            return Err(AppError::Validation(
                DomainError::PasswordTooShort.to_string(),
            ));
        }

        match self.user_repo.find_by_email(email_vo.as_str()).await {
            Ok(_) => return Err(AppError::Conflict("email already registered".into())),
            Err(crate::ports::repositories::RepositoryError::NotFound) => {}
            Err(e) => return Err(AppError::from(e)),
        }

        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::Internal(e.to_string()))?
            .to_string();

        let user = self
            .user_repo
            .insert(email_vo.as_str(), &hash, &name)
            .await
            .map_err(AppError::from)?;

        issue_session(self.sessions.as_ref(), self.auth.as_ref(), user).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Mutex;
    use uuid::Uuid;

    use crate::domain::user::{
        entity::User,
        value_objects::{Email, PasswordHash},
    };
    use crate::ports::session::{NewSession, RotateOutcome, SessionError, SessionResult};

    struct MockUserRepo {
        existing_emails: Vec<String>,
        inserted: Mutex<Vec<String>>,
    }

    #[async_trait::async_trait]
    impl crate::ports::repositories::UserRepository for MockUserRepo {
        async fn find_by_email(
            &self,
            email: &str,
        ) -> crate::ports::repositories::RepositoryResult<User> {
            if self.existing_emails.contains(&email.to_string()) {
                Ok(User {
                    id: Uuid::new_v4(),
                    email: Email::new(email.to_string()).unwrap(),
                    password_hash: PasswordHash::from_hash("hash".into()),
                    name: "Existing".into(),
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
            email: &str,
            _hash: &str,
            _name: &str,
        ) -> crate::ports::repositories::RepositoryResult<User> {
            self.inserted.lock().unwrap().push(email.to_string());
            Ok(User {
                id: Uuid::new_v4(),
                email: Email::new(email.to_string()).unwrap(),
                password_hash: PasswordHash::from_hash("hash".into()),
                name: "New User".into(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            })
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

    fn make_uc(existing_emails: Vec<String>) -> RegisterUseCase {
        RegisterUseCase {
            user_repo: Arc::new(MockUserRepo {
                existing_emails,
                inserted: Mutex::new(vec![]),
            }),
            auth: Arc::new(MockAuth),
            sessions: Arc::new(MockSessions),
            access_ttl_secs: 900,
        }
    }

    #[tokio::test]
    async fn register_returns_session_on_success() {
        let uc = make_uc(vec![]);
        let result = uc
            .execute(
                "alice@example.com".into(),
                "password123".into(),
                "Alice".into(),
            )
            .await;
        let outcome = result.expect("expected Ok");
        assert_eq!(outcome.session.access_token, "mock.access.token");
        assert_eq!(outcome.session.sid, outcome.session.family_id);
        // Fix 4: the user is returned alongside the session (no 2nd DB read).
        assert_eq!(outcome.user.email.as_str(), "alice@example.com");
    }

    #[tokio::test]
    async fn register_rejects_duplicate_email() {
        let uc = make_uc(vec!["alice@example.com".into()]);
        let result = uc
            .execute(
                "alice@example.com".into(),
                "password123".into(),
                "Alice".into(),
            )
            .await;
        assert!(matches!(result, Err(AppError::Conflict(_))));
    }

    #[tokio::test]
    async fn register_rejects_short_password() {
        let uc = make_uc(vec![]);
        let result = uc
            .execute("alice@example.com".into(), "short".into(), "Alice".into())
            .await;
        assert!(matches!(result, Err(AppError::Validation(_))));
    }
}
