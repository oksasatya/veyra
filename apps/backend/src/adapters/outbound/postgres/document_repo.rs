use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::document::entity::{DocType, Document},
    ports::repositories::{CreateDocumentParams, DocumentRepository, RepositoryError, RepositoryResult},
};

use super::models::DocumentRow;

/// Column list for SELECT (no table prefix — used in single-table RETURNING).
const SELECT_COLS: &str =
    "id, vehicle_id, doc_type, title, expiry_date, file_url, notes, created_at";

/// Column list with `d.` prefix — used in JOINed queries where vehicles also
/// has overlapping column names (e.g., `id`).
const SELECT_COLS_PREFIXED: &str =
    "d.id, d.vehicle_id, d.doc_type, d.title, d.expiry_date, d.file_url, d.notes, d.created_at";

/// Postgres implementation of [`DocumentRepository`].
pub struct PgDocumentRepo {
    pool: PgPool,
}

impl PgDocumentRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Converts a `sqlx::Error` to `RepositoryError`, keeping the ports layer
/// free of sqlx (layer-boundary rule).
fn map_pg_err(e: sqlx::Error) -> RepositoryError {
    RepositoryError::Database(e.to_string())
}

/// Maps a `DocumentRow` (sqlx) to a domain `Document`.
///
/// `doc_type` is stored as TEXT in Postgres. Defaults to `Other` if the stored
/// value is somehow unrecognised — avoids a hard crash on a bad DB row.
fn row_to_document(r: DocumentRow) -> Document {
    Document {
        id: r.id,
        vehicle_id: r.vehicle_id,
        doc_type: DocType::parse(&r.doc_type).unwrap_or(DocType::Other),
        title: r.title,
        expiry_date: r.expiry_date,
        file_url: r.file_url,
        notes: r.notes,
        created_at: r.created_at,
    }
}

#[async_trait]
impl DocumentRepository for PgDocumentRepo {
    /// Lists documents for a vehicle, scoped via a JOIN to ensure the vehicle
    /// belongs to `user_id` — defence-in-depth even though the use case already
    /// checks ownership. Ordered by `created_at DESC`.
    async fn list_by_vehicle(
        &self,
        vehicle_id: Uuid,
        user_id: Uuid,
    ) -> RepositoryResult<Vec<Document>> {
        let query = format!(
            "SELECT {SELECT_COLS_PREFIXED} \
             FROM vehicle_documents d \
             JOIN vehicles v ON v.id = d.vehicle_id \
             WHERE d.vehicle_id = $1 AND v.user_id = $2 \
             ORDER BY d.created_at DESC"
        );
        let rows = sqlx::query_as::<_, DocumentRow>(&query)
            .bind(vehicle_id)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(map_pg_err)?;

        Ok(rows.into_iter().map(row_to_document).collect())
    }

    async fn insert(&self, p: CreateDocumentParams) -> RepositoryResult<Document> {
        let query = format!(
            r#"INSERT INTO vehicle_documents
               (vehicle_id, doc_type, title, expiry_date, file_url, notes)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING {SELECT_COLS}"#
        );
        let row = sqlx::query_as::<_, DocumentRow>(&query)
            .bind(p.vehicle_id)
            .bind(&p.doc_type)
            .bind(&p.title)
            .bind(p.expiry_date)
            .bind(p.file_url.as_deref())
            .bind(p.notes.as_deref())
            .fetch_one(&self.pool)
            .await
            .map_err(map_pg_err)?;

        Ok(row_to_document(row))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use testcontainers_modules::postgres::Postgres;
    use testcontainers_modules::testcontainers::runners::AsyncRunner;
    use testcontainers_modules::testcontainers::RunnableImage;

    /// Spins up Postgres 16-alpine, runs all migrations, returns pool + container.
    /// Caller must bind `_container` to keep it alive for the pool's lifetime.
    async fn setup_pool() -> (
        PgPool,
        testcontainers_modules::testcontainers::ContainerAsync<Postgres>,
    ) {
        let image = RunnableImage::from(Postgres::default()).with_tag("16-alpine");
        let container = image.start().await;
        let port = container.get_host_port_ipv4(5432).await;
        let url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");
        let pool = PgPool::connect(&url).await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        (pool, container)
    }

    /// Inserts a bare user row and returns its id.
    async fn insert_user(pool: &PgPool) -> Uuid {
        let row: (Uuid,) = sqlx::query_as(
            "INSERT INTO users (email, password_hash, name) VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(format!("user_{}@test.com", Uuid::new_v4()))
        .bind("hash")
        .bind("Test User")
        .fetch_one(pool)
        .await
        .unwrap();
        row.0
    }

    /// Inserts a bare vehicle row owned by `user_id` and returns its id.
    async fn insert_vehicle(pool: &PgPool, user_id: Uuid) -> Uuid {
        let row: (Uuid,) = sqlx::query_as(
            "INSERT INTO vehicles (user_id, brand, model, year, plate_number, fuel_type, current_odometer) \
             VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
        )
        .bind(user_id)
        .bind("Toyota")
        .bind("Avanza")
        .bind(2020_i16)
        .bind(format!("B {} TST", Uuid::new_v4().simple()))
        .bind("petrol")
        .bind(0_i32)
        .fetch_one(pool)
        .await
        .unwrap();
        row.0
    }

    fn make_params(vehicle_id: Uuid) -> CreateDocumentParams {
        CreateDocumentParams {
            vehicle_id,
            doc_type: "stnk".into(),
            title: "STNK 2026".into(),
            expiry_date: Some(NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()),
            file_url: Some("https://storage.example.com/stnk.pdf".into()),
            notes: None,
        }
    }

    #[tokio::test]
    async fn insert_and_list_by_vehicle() {
        let (pool, _container) = setup_pool().await;
        let user_id = insert_user(&pool).await;
        let vehicle_id = insert_vehicle(&pool, user_id).await;
        let repo = PgDocumentRepo::new(pool);

        let doc = repo.insert(make_params(vehicle_id)).await.unwrap();

        assert_eq!(doc.doc_type, DocType::Stnk);
        assert_eq!(doc.title, "STNK 2026");
        assert_eq!(
            doc.file_url.as_deref(),
            Some("https://storage.example.com/stnk.pdf")
        );

        let docs = repo.list_by_vehicle(vehicle_id, user_id).await.unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].title, "STNK 2026");
    }

    #[tokio::test]
    async fn list_by_vehicle_with_wrong_user_returns_empty() {
        let (pool, _container) = setup_pool().await;
        let owner_id = insert_user(&pool).await;
        let intruder_id = insert_user(&pool).await;
        let vehicle_id = insert_vehicle(&pool, owner_id).await;
        let repo = PgDocumentRepo::new(pool);

        repo.insert(make_params(vehicle_id)).await.unwrap();

        // Querying as a different user should return nothing (JOIN filters it out)
        let docs = repo
            .list_by_vehicle(vehicle_id, intruder_id)
            .await
            .unwrap();
        assert_eq!(docs.len(), 0);
    }

    #[tokio::test]
    async fn insert_without_optional_fields() {
        let (pool, _container) = setup_pool().await;
        let user_id = insert_user(&pool).await;
        let vehicle_id = insert_vehicle(&pool, user_id).await;
        let repo = PgDocumentRepo::new(pool);

        let params = CreateDocumentParams {
            vehicle_id,
            doc_type: "insurance".into(),
            title: "Insurance 2026".into(),
            expiry_date: None,
            file_url: None,
            notes: None,
        };
        let doc = repo.insert(params).await.unwrap();

        assert_eq!(doc.doc_type, DocType::Insurance);
        assert!(doc.expiry_date.is_none());
        assert!(doc.file_url.is_none());
    }

    #[tokio::test]
    async fn list_returns_documents_ordered_by_created_at_desc() {
        let (pool, _container) = setup_pool().await;
        let user_id = insert_user(&pool).await;
        let vehicle_id = insert_vehicle(&pool, user_id).await;
        let repo = PgDocumentRepo::new(pool);

        // Insert two documents sequentially (created_at will differ by DB clock)
        let first = CreateDocumentParams {
            vehicle_id,
            doc_type: "bpkb".into(),
            title: "BPKB".into(),
            expiry_date: None,
            file_url: None,
            notes: None,
        };
        let second = CreateDocumentParams {
            vehicle_id,
            doc_type: "stnk".into(),
            title: "STNK".into(),
            expiry_date: None,
            file_url: None,
            notes: None,
        };
        repo.insert(first).await.unwrap();
        // Small sleep to ensure distinct created_at timestamps
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        repo.insert(second).await.unwrap();

        let docs = repo.list_by_vehicle(vehicle_id, user_id).await.unwrap();
        assert_eq!(docs.len(), 2);
        // Most recent first
        assert_eq!(docs[0].title, "STNK");
        assert_eq!(docs[1].title, "BPKB");
    }
}
