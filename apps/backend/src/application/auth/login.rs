use std::sync::Arc;

use argon2::{Argon2, PasswordHash, PasswordVerifier};

use crate::{
    application::errors::AppError,
    domain::user::value_objects::Email,
    ports::{
        auth::AuthPort,
        repositories::{RepositoryError, UserRepository},
    },
};

pub struct LoginUseCase {
    pub user_repo: Arc<dyn UserRepository>,
    pub auth: Arc<dyn AuthPort>,
}

impl LoginUseCase {
    pub async fn execute(&self, email: String, password: String) -> Result<String, AppError> {
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

        self.auth
            .sign_token(user.id)
            .map_err(|e| AppError::Internal(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use argon2::{
        password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
        Argon2 as Argon2Hash,
    };
    use uuid::Uuid;

    use crate::domain::user::{
        entity::User,
        value_objects::{Email, PasswordHash as DomainPasswordHash},
    };

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
        fn sign_token(&self, _id: Uuid) -> Result<String, crate::ports::auth::AuthError> {
            Ok("mock.jwt.token".into())
        }
        fn verify_token(&self, _token: &str) -> Result<Uuid, crate::ports::auth::AuthError> {
            Ok(Uuid::new_v4())
        }
    }

    fn make_hash(password: &str) -> String {
        let salt = SaltString::generate(&mut OsRng);
        Argon2Hash::default()
            .hash_password(password.as_bytes(), &salt)
            .unwrap()
            .to_string()
    }

    #[tokio::test]
    async fn login_returns_jwt_on_correct_credentials() {
        let hash = make_hash("password123");
        let repo = Arc::new(MockUserRepo {
            email: "alice@example.com".into(),
            password_hash: hash,
        });
        let auth = Arc::new(MockAuth);
        let uc = LoginUseCase {
            user_repo: repo,
            auth,
        };
        let result = uc
            .execute("alice@example.com".into(), "password123".into())
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "mock.jwt.token");
    }

    #[tokio::test]
    async fn login_rejects_wrong_password() {
        let hash = make_hash("password123");
        let repo = Arc::new(MockUserRepo {
            email: "alice@example.com".into(),
            password_hash: hash,
        });
        let auth = Arc::new(MockAuth);
        let uc = LoginUseCase {
            user_repo: repo,
            auth,
        };
        let result = uc
            .execute("alice@example.com".into(), "wrongpassword".into())
            .await;
        assert!(matches!(result, Err(AppError::Unauthorized)));
    }

    #[tokio::test]
    async fn login_rejects_unknown_email() {
        let hash = make_hash("password123");
        let repo = Arc::new(MockUserRepo {
            email: "alice@example.com".into(),
            password_hash: hash,
        });
        let auth = Arc::new(MockAuth);
        let uc = LoginUseCase {
            user_repo: repo,
            auth,
        };
        let result = uc
            .execute("nobody@example.com".into(), "password123".into())
            .await;
        assert!(matches!(result, Err(AppError::Unauthorized)));
    }
}
