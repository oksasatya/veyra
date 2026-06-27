use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::service_record::entity::ServiceRecord,
    ports::repositories::{
        CreateServiceRecordParams, RepositoryError, RepositoryResult, ServiceRecordRepository,
    },
};

use super::models::ServiceRecordRow;

/// Column list for SELECT (no table prefix — used in single-table RETURNING).
const SELECT_COLS: &str =
    "id, vehicle_id, service_date, odometer, description, workshop, cost, notes, created_at";

/// Column list with `sr.` prefix — used in JOINed queries where vehicles also
/// has overlapping column names (e.g., `notes`).
const SELECT_COLS_PREFIXED: &str =
    "sr.id, sr.vehicle_id, sr.service_date, sr.odometer, sr.description, \
     sr.workshop, sr.cost, sr.notes, sr.created_at";

/// Postgres implementation of [`ServiceRecordRepository`].
pub struct PgServiceRecordRepo {
    pool: PgPool,
}

impl PgServiceRecordRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Converts a `sqlx::Error` to `RepositoryError`, keeping the ports layer
/// free of sqlx (layer-boundary rule).
fn map_pg_err(e: sqlx::Error) -> RepositoryError {
    RepositoryError::Database(e.to_string())
}

/// Maps a `ServiceRecordRow` (sqlx) to a domain `ServiceRecord`.
///
/// `odometer` is stored as `INTEGER` (i32) in Postgres but exposed as `u32`
/// in the domain — cast is safe because we validate non-negative on write.
fn row_to_entity(r: ServiceRecordRow) -> ServiceRecord {
    ServiceRecord {
        id: r.id,
        vehicle_id: r.vehicle_id,
        service_date: r.service_date,
        odometer: r.odometer as u32,
        description: r.description,
        workshop: r.workshop,
        cost: r.cost,
        notes: r.notes,
        created_at: r.created_at,
    }
}

#[async_trait]
impl ServiceRecordRepository for PgServiceRecordRepo {
    /// Lists service records for a vehicle, scoped via a JOIN to ensure the
    /// vehicle belongs to `user_id` — defence-in-depth even though the use
    /// case already checks ownership.
    async fn list_by_vehicle(
        &self,
        vehicle_id: Uuid,
        user_id: Uuid,
    ) -> RepositoryResult<Vec<ServiceRecord>> {
        let query = format!(
            "SELECT {SELECT_COLS_PREFIXED} \
             FROM service_records sr \
             JOIN vehicles v ON v.id = sr.vehicle_id \
             WHERE sr.vehicle_id = $1 AND v.user_id = $2 \
             ORDER BY sr.service_date DESC"
        );
        let rows = sqlx::query_as::<_, ServiceRecordRow>(&query)
            .bind(vehicle_id)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(map_pg_err)?;

        Ok(rows.into_iter().map(row_to_entity).collect())
    }

    async fn insert(&self, p: CreateServiceRecordParams) -> RepositoryResult<ServiceRecord> {
        let query = format!(
            r#"INSERT INTO service_records
               (vehicle_id, service_date, odometer, description, workshop, cost, notes)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               RETURNING {SELECT_COLS}"#
        );
        let row = sqlx::query_as::<_, ServiceRecordRow>(&query)
            .bind(p.vehicle_id)
            .bind(p.service_date)
            .bind(p.odometer as i32)
            .bind(&p.description)
            .bind(&p.workshop)
            .bind(p.cost)
            .bind(&p.notes)
            .fetch_one(&self.pool)
            .await
            .map_err(map_pg_err)?;

        Ok(row_to_entity(row))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
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

    fn make_params(vehicle_id: Uuid) -> CreateServiceRecordParams {
        CreateServiceRecordParams {
            vehicle_id,
            service_date: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            odometer: 5_000,
            description: "Oil change".into(),
            workshop: Some("Fast Lube".into()),
            cost: Some(Decimal::new(15000, 2)),
            notes: None,
        }
    }

    #[tokio::test]
    async fn insert_and_list_by_vehicle() {
        let (pool, _container) = setup_pool().await;
        let user_id = insert_user(&pool).await;
        let vehicle_id = insert_vehicle(&pool, user_id).await;
        let repo = PgServiceRecordRepo::new(pool);

        repo.insert(make_params(vehicle_id)).await.unwrap();

        let records = repo.list_by_vehicle(vehicle_id, user_id).await.unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].description, "Oil change");
        assert_eq!(records[0].odometer, 5_000);
        assert_eq!(records[0].workshop.as_deref(), Some("Fast Lube"));
    }

    #[tokio::test]
    async fn list_by_vehicle_with_wrong_user_returns_empty() {
        let (pool, _container) = setup_pool().await;
        let owner_id = insert_user(&pool).await;
        let intruder_id = insert_user(&pool).await;
        let vehicle_id = insert_vehicle(&pool, owner_id).await;
        let repo = PgServiceRecordRepo::new(pool);

        repo.insert(make_params(vehicle_id)).await.unwrap();

        // Querying as a different user should return nothing (JOIN filters it out)
        let records = repo
            .list_by_vehicle(vehicle_id, intruder_id)
            .await
            .unwrap();
        assert_eq!(records.len(), 0);
    }

    #[tokio::test]
    async fn insert_stores_optional_fields_correctly() {
        let (pool, _container) = setup_pool().await;
        let user_id = insert_user(&pool).await;
        let vehicle_id = insert_vehicle(&pool, user_id).await;
        let repo = PgServiceRecordRepo::new(pool);

        let params = CreateServiceRecordParams {
            vehicle_id,
            service_date: NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            odometer: 10_000,
            description: "Brake pad replacement".into(),
            workshop: None,
            cost: None,
            notes: Some("Both front pads".into()),
        };
        let record = repo.insert(params).await.unwrap();

        assert!(record.workshop.is_none());
        assert!(record.cost.is_none());
        assert_eq!(record.notes.as_deref(), Some("Both front pads"));
    }

    #[tokio::test]
    async fn list_returns_multiple_records_ordered_by_date_desc() {
        let (pool, _container) = setup_pool().await;
        let user_id = insert_user(&pool).await;
        let vehicle_id = insert_vehicle(&pool, user_id).await;
        let repo = PgServiceRecordRepo::new(pool);

        // Insert two records: older first, newer second
        let older = CreateServiceRecordParams {
            vehicle_id,
            service_date: NaiveDate::from_ymd_opt(2025, 6, 1).unwrap(),
            odometer: 3_000,
            description: "Older service".into(),
            workshop: None,
            cost: None,
            notes: None,
        };
        let newer = CreateServiceRecordParams {
            vehicle_id,
            service_date: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            odometer: 5_000,
            description: "Newer service".into(),
            workshop: None,
            cost: None,
            notes: None,
        };
        repo.insert(older).await.unwrap();
        repo.insert(newer).await.unwrap();

        let records = repo.list_by_vehicle(vehicle_id, user_id).await.unwrap();
        assert_eq!(records.len(), 2);
        // Most recent date first
        assert_eq!(records[0].description, "Newer service");
        assert_eq!(records[1].description, "Older service");
    }
}
