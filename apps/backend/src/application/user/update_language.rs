use std::sync::Arc;

use uuid::Uuid;

use crate::{
    application::errors::AppError,
    domain::user::{entity::User, value_objects::Language},
    ports::repositories::UserRepository,
};

/// Validates a language code and persists it as the user's preferred language.
pub struct UpdateLanguageUseCase {
    pub user_repo: Arc<dyn UserRepository>,
}

impl UpdateLanguageUseCase {
    /// Parse + validate `language` (`"en"` / `"id"`), then persist it.
    ///
    /// Returns `AppError::Validation` (code `INVALID_LANGUAGE`) for an unsupported
    /// code, or `AppError::NotFound` if the user no longer exists.
    pub async fn execute(&self, user_id: Uuid, language: &str) -> Result<User, AppError> {
        let lang = Language::parse(language)?;
        let user = self
            .user_repo
            .update_language(user_id, lang.as_str())
            .await?;
        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::{
            error_code::ErrorCode,
            user::{
                entity::User,
                value_objects::{Email, PasswordHash},
            },
        },
        ports::repositories::{RepositoryError, RepositoryResult},
    };
    use async_trait::async_trait;

    struct FakeUserRepo;

    fn fake_user(language: Language) -> User {
        User {
            id: Uuid::new_v4(),
            email: Email::new("alice@example.com".into()).unwrap(),
            password_hash: PasswordHash::from_hash("hash".into()),
            name: "Alice".into(),
            preferred_language: language,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[async_trait]
    impl UserRepository for FakeUserRepo {
        async fn find_by_email(&self, _email: &str) -> RepositoryResult<User> {
            Err(RepositoryError::NotFound)
        }
        async fn find_by_id(&self, _id: Uuid) -> RepositoryResult<User> {
            Err(RepositoryError::NotFound)
        }
        async fn insert(&self, _e: &str, _h: &str, _n: &str) -> RepositoryResult<User> {
            Err(RepositoryError::NotFound)
        }
        async fn update_language(&self, _id: Uuid, language: &str) -> RepositoryResult<User> {
            Ok(fake_user(Language::parse(language).unwrap_or_default()))
        }
    }

    fn make_uc() -> UpdateLanguageUseCase {
        UpdateLanguageUseCase {
            user_repo: Arc::new(FakeUserRepo),
        }
    }

    #[tokio::test]
    async fn valid_language_is_persisted() {
        let user = make_uc().execute(Uuid::new_v4(), "id").await.unwrap();
        assert_eq!(user.preferred_language, Language::Id);
    }

    #[tokio::test]
    async fn unsupported_language_is_rejected() {
        let err = make_uc().execute(Uuid::new_v4(), "fr").await.unwrap_err();
        assert_eq!(err.code(), ErrorCode::InvalidLanguage);
    }
}
