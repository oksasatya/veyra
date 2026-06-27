use std::sync::Arc;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};

use crate::{
    application::errors::AppError,
    domain::{errors::DomainError, user::value_objects::Email},
    ports::{auth::AuthPort, repositories::UserRepository},
};

pub struct RegisterUseCase {
    pub user_repo: Arc<dyn UserRepository>,
    pub auth: Arc<dyn AuthPort>,
}

impl RegisterUseCase {
    pub async fn execute(
        &self,
        email: String,
        password: String,
        name: String,
    ) -> Result<String, AppError> {
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

        self.auth
            .sign_token(user.id)
            .map_err(|e| AppError::Internal(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use uuid::Uuid;

    use crate::domain::user::{
        entity::User,
        value_objects::{Email, PasswordHash},
    };

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
        fn sign_token(&self, _id: Uuid) -> Result<String, crate::ports::auth::AuthError> {
            Ok("mock.jwt.token".into())
        }
        fn verify_token(&self, _token: &str) -> Result<Uuid, crate::ports::auth::AuthError> {
            Ok(Uuid::new_v4())
        }
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

    #[tokio::test]
    async fn register_returns_jwt_on_success() {
        let repo = Arc::new(MockUserRepo {
            existing_emails: vec![],
            inserted: Mutex::new(vec![]),
        });
        let auth = Arc::new(MockAuth);
        let uc = RegisterUseCase {
            user_repo: repo,
            auth,
        };
        let result = uc
            .execute(
                "alice@example.com".into(),
                "password123".into(),
                "Alice".into(),
            )
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "mock.jwt.token");
    }

    #[tokio::test]
    async fn register_rejects_duplicate_email() {
        let repo = Arc::new(MockUserRepo {
            existing_emails: vec!["alice@example.com".into()],
            inserted: Mutex::new(vec![]),
        });
        let auth = Arc::new(MockAuth);
        let uc = RegisterUseCase {
            user_repo: repo,
            auth,
        };
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
        let repo = Arc::new(MockUserRepo {
            existing_emails: vec![],
            inserted: Mutex::new(vec![]),
        });
        let auth = Arc::new(MockAuth);
        let uc = RegisterUseCase {
            user_repo: repo,
            auth,
        };
        let result = uc
            .execute("alice@example.com".into(), "short".into(), "Alice".into())
            .await;
        assert!(matches!(result, Err(AppError::Validation(_))));
    }
}
