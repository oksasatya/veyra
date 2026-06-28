use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::user::{
        entity::User,
        value_objects::{Email, Language, PasswordHash},
    },
    ports::repositories::{RepositoryError, RepositoryResult, UserRepository},
};

use super::models::UserRow;

const EMAIL_CONSTRAINT: &str = "users_email_key";

/// Postgres implementation of [`UserRepository`].
pub struct PgUserRepo {
    pool: PgPool,
}

impl PgUserRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Maps a `sqlx::Error` to `RepositoryError`, detecting the email uniqueness
/// constraint so callers get `Conflict` rather than a generic `Database` error.
fn map_pg_err(e: sqlx::Error) -> RepositoryError {
    if let sqlx::Error::Database(ref db_err) = e {
        if db_err.constraint() == Some(EMAIL_CONSTRAINT) {
            return RepositoryError::Conflict("email already registered".into());
        }
    }
    RepositoryError::Database(e.to_string())
}

/// Converts a raw `UserRow` (sqlx) into a domain `User`.
///
/// # Errors
/// Returns `RepositoryError::Database` if the stored email fails domain
/// validation — which would indicate data corruption since validation is
/// enforced on write.
fn row_to_user(row: UserRow) -> Result<User, RepositoryError> {
    let email = Email::new(row.email).map_err(|e| RepositoryError::Database(e.to_string()))?;
    // The DB CHECK constraint guarantees a valid code; default defensively if not.
    let preferred_language = Language::parse(&row.preferred_language).unwrap_or_default();
    Ok(User {
        id: row.id,
        email,
        password_hash: PasswordHash::from_hash(row.password_hash),
        name: row.name,
        preferred_language,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

#[async_trait]
impl UserRepository for PgUserRepo {
    async fn find_by_email(&self, email: &str) -> RepositoryResult<User> {
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, email, password_hash, name, preferred_language, created_at, updated_at \
             FROM users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?
        .ok_or(RepositoryError::NotFound)?;

        row_to_user(row)
    }

    async fn find_by_id(&self, id: Uuid) -> RepositoryResult<User> {
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, email, password_hash, name, preferred_language, created_at, updated_at \
             FROM users WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?
        .ok_or(RepositoryError::NotFound)?;

        row_to_user(row)
    }

    async fn insert(&self, email: &str, password_hash: &str, name: &str) -> RepositoryResult<User> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"INSERT INTO users (email, password_hash, name)
               VALUES ($1, $2, $3)
               RETURNING id, email, password_hash, name, preferred_language, created_at, updated_at"#,
        )
        .bind(email)
        .bind(password_hash)
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map_err(map_pg_err)?;

        row_to_user(row)
    }

    async fn update_language(&self, id: Uuid, language: &str) -> RepositoryResult<User> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"UPDATE users
               SET preferred_language = $2, updated_at = NOW()
               WHERE id = $1
               RETURNING id, email, password_hash, name, preferred_language, created_at, updated_at"#,
        )
        .bind(id)
        .bind(language)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?
        .ok_or(RepositoryError::NotFound)?;

        row_to_user(row)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use testcontainers_modules::postgres::Postgres;
    use testcontainers_modules::testcontainers::{runners::AsyncRunner, ImageExt};

    /// Spin up a fresh Postgres 16 container (gen_random_uuid() built-in),
    /// run all migrations, and return the pool with the container handle.
    /// The container must outlive the pool — callers bind it with `_container`.
    async fn setup_pool() -> (
        PgPool,
        testcontainers_modules::testcontainers::ContainerAsync<Postgres>,
    ) {
        let container = Postgres::default()
            .with_tag("16-alpine")
            .start()
            .await
            .unwrap();
        let port = container.get_host_port_ipv4(5432).await.unwrap();
        let url = format!("postgres://postgres:postgres@127.0.0.1:{}/postgres", port);
        let pool = PgPool::connect(&url).await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        (pool, container)
    }

    #[tokio::test]
    async fn insert_and_find_by_email() {
        let (pool, _container) = setup_pool().await;
        let repo = PgUserRepo::new(pool);

        let user = repo
            .insert("bob@example.com", "hashed_password", "Bob")
            .await
            .unwrap();

        assert_eq!(user.email.as_str(), "bob@example.com");
        assert_eq!(user.name, "Bob");

        let found = repo.find_by_email("bob@example.com").await.unwrap();
        assert_eq!(found.id, user.id);
        assert_eq!(found.name, "Bob");
    }

    #[tokio::test]
    async fn find_by_id_roundtrip() {
        let (pool, _container) = setup_pool().await;
        let repo = PgUserRepo::new(pool);

        let created = repo
            .insert("alice@example.com", "hash_abc", "Alice")
            .await
            .unwrap();

        let found = repo.find_by_id(created.id).await.unwrap();
        assert_eq!(found.id, created.id);
        assert_eq!(found.email.as_str(), "alice@example.com");
    }

    #[tokio::test]
    async fn find_by_email_not_found_returns_error() {
        let (pool, _container) = setup_pool().await;
        let repo = PgUserRepo::new(pool);
        let result = repo.find_by_email("nobody@example.com").await;
        assert!(matches!(result, Err(RepositoryError::NotFound)));
    }

    #[tokio::test]
    async fn find_by_id_not_found_returns_error() {
        let (pool, _container) = setup_pool().await;
        let repo = PgUserRepo::new(pool);
        let result = repo.find_by_id(Uuid::new_v4()).await;
        assert!(matches!(result, Err(RepositoryError::NotFound)));
    }

    #[tokio::test]
    async fn duplicate_email_returns_conflict() {
        let (pool, _container) = setup_pool().await;
        let repo = PgUserRepo::new(pool);
        repo.insert("dup@example.com", "hash", "First")
            .await
            .unwrap();
        let result = repo.insert("dup@example.com", "hash", "Second").await;
        assert!(matches!(result, Err(RepositoryError::Conflict(_))));
    }
}
