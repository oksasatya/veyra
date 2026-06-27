use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::fuel_log::entity::FuelLog,
    ports::repositories::{
        CreateFuelLogParams, FuelLogRepository, RepositoryError, RepositoryResult,
    },
};

use super::models::FuelLogRow;

/// Column list for SELECT (no table prefix — used in single-table queries).
const SELECT_COLS: &str =
    "id, vehicle_id, log_date, odometer, liters, price_per_liter, total_cost, \
     station, is_full_tank, created_at";

/// Column list with `fl.` prefix — used in JOINed queries where vehicles also
/// has overlapping column names (e.g., `id`).
const SELECT_COLS_PREFIXED: &str =
    "fl.id, fl.vehicle_id, fl.log_date, fl.odometer, fl.liters, fl.price_per_liter, \
     fl.total_cost, fl.station, fl.is_full_tank, fl.created_at";

/// Postgres implementation of [`FuelLogRepository`].
pub struct PgFuelLogRepo {
    pool: PgPool,
}

impl PgFuelLogRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Converts a `sqlx::Error` to `RepositoryError`, keeping the ports layer
/// free of sqlx (layer-boundary rule).
fn map_pg_err(e: sqlx::Error) -> RepositoryError {
    RepositoryError::Database(e.to_string())
}

/// Maps a `FuelLogRow` (sqlx) to a domain `FuelLog`.
///
/// `odometer` is stored as `INTEGER` (i32) in Postgres but exposed as `u32`
/// in the domain — cast is safe because we validate non-negative on write.
fn row_to_fuel_log(r: FuelLogRow) -> FuelLog {
    FuelLog {
        id: r.id,
        vehicle_id: r.vehicle_id,
        log_date: r.log_date,
        odometer: r.odometer as u32,
        liters: r.liters,
        price_per_liter: r.price_per_liter,
        total_cost: r.total_cost,
        station: r.station,
        is_full_tank: r.is_full_tank,
        created_at: r.created_at,
    }
}

#[async_trait]
impl FuelLogRepository for PgFuelLogRepo {
    /// Lists fuel logs for a vehicle, scoped via a JOIN to ensure the vehicle
    /// belongs to `user_id` — defence-in-depth even though the use case already
    /// checks ownership.
    async fn list_by_vehicle(
        &self,
        vehicle_id: Uuid,
        user_id: Uuid,
    ) -> RepositoryResult<Vec<FuelLog>> {
        let query = format!(
            "SELECT {SELECT_COLS_PREFIXED} \
             FROM fuel_logs fl \
             JOIN vehicles v ON v.id = fl.vehicle_id \
             WHERE fl.vehicle_id = $1 AND v.user_id = $2 \
             ORDER BY fl.log_date DESC"
        );
        let rows = sqlx::query_as::<_, FuelLogRow>(&query)
            .bind(vehicle_id)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(map_pg_err)?;

        Ok(rows.into_iter().map(row_to_fuel_log).collect())
    }

    async fn insert(&self, p: CreateFuelLogParams) -> RepositoryResult<FuelLog> {
        let query = format!(
            r#"INSERT INTO fuel_logs
               (vehicle_id, log_date, odometer, liters, price_per_liter, station, is_full_tank)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               RETURNING {SELECT_COLS}"#
        );
        let row = sqlx::query_as::<_, FuelLogRow>(&query)
            .bind(p.vehicle_id)
            .bind(p.log_date)
            .bind(p.odometer as i32)
            .bind(p.liters)
            .bind(p.price_per_liter)
            .bind(&p.station)
            .bind(p.is_full_tank)
            .fetch_one(&self.pool)
            .await
            .map_err(map_pg_err)?;

        Ok(row_to_fuel_log(row))
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

    fn make_params(vehicle_id: Uuid) -> CreateFuelLogParams {
        CreateFuelLogParams {
            vehicle_id,
            log_date: NaiveDate::from_ymd_opt(2026, 1, 20).unwrap(),
            odometer: 10_000,
            liters: Decimal::new(400, 1),              // 40.0
            price_per_liter: Decimal::new(100_000, 1), // 10000.0
            station: Some("Shell".into()),
            is_full_tank: true,
        }
    }

    #[tokio::test]
    async fn insert_and_list_by_vehicle() {
        let (pool, _container) = setup_pool().await;
        let user_id = insert_user(&pool).await;
        let vehicle_id = insert_vehicle(&pool, user_id).await;
        let repo = PgFuelLogRepo::new(pool);

        let log = repo.insert(make_params(vehicle_id)).await.unwrap();

        // total_cost is GENERATED ALWAYS AS (liters * price_per_liter) STORED
        assert_eq!(log.liters, Decimal::new(400, 1));
        assert_eq!(log.price_per_liter, Decimal::new(100_000, 1));
        // 40.0 * 10000.0 = 400000.00 — Postgres computes this
        assert_eq!(log.total_cost.to_string(), "400000.00");

        let logs = repo.list_by_vehicle(vehicle_id, user_id).await.unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].odometer, 10_000);
        assert_eq!(logs[0].station.as_deref(), Some("Shell"));
        assert!(logs[0].is_full_tank);
    }

    #[tokio::test]
    async fn list_by_vehicle_with_wrong_user_returns_empty() {
        let (pool, _container) = setup_pool().await;
        let owner_id = insert_user(&pool).await;
        let intruder_id = insert_user(&pool).await;
        let vehicle_id = insert_vehicle(&pool, owner_id).await;
        let repo = PgFuelLogRepo::new(pool);

        repo.insert(make_params(vehicle_id)).await.unwrap();

        // Querying as a different user should return nothing (JOIN filters it out)
        let logs = repo.list_by_vehicle(vehicle_id, intruder_id).await.unwrap();
        assert_eq!(logs.len(), 0);
    }

    #[tokio::test]
    async fn insert_stores_optional_fields_correctly() {
        let (pool, _container) = setup_pool().await;
        let user_id = insert_user(&pool).await;
        let vehicle_id = insert_vehicle(&pool, user_id).await;
        let repo = PgFuelLogRepo::new(pool);

        let params = CreateFuelLogParams {
            vehicle_id,
            log_date: NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            odometer: 15_000,
            liters: Decimal::new(350, 1),             // 35.0
            price_per_liter: Decimal::new(95_000, 1), // 9500.0
            station: None,
            is_full_tank: false,
        };
        let log = repo.insert(params).await.unwrap();

        assert!(log.station.is_none());
        assert!(!log.is_full_tank);
    }

    #[tokio::test]
    async fn list_returns_multiple_logs_ordered_by_date_desc() {
        let (pool, _container) = setup_pool().await;
        let user_id = insert_user(&pool).await;
        let vehicle_id = insert_vehicle(&pool, user_id).await;
        let repo = PgFuelLogRepo::new(pool);

        // Insert two logs: older first, newer second
        let older = CreateFuelLogParams {
            vehicle_id,
            log_date: NaiveDate::from_ymd_opt(2025, 6, 1).unwrap(),
            odometer: 5_000,
            liters: Decimal::new(300, 1),
            price_per_liter: Decimal::new(90_000, 1),
            station: Some("Pertamina".into()),
            is_full_tank: true,
        };
        let newer = CreateFuelLogParams {
            vehicle_id,
            log_date: NaiveDate::from_ymd_opt(2026, 1, 20).unwrap(),
            odometer: 10_000,
            liters: Decimal::new(400, 1),
            price_per_liter: Decimal::new(100_000, 1),
            station: Some("Shell".into()),
            is_full_tank: false,
        };
        repo.insert(older).await.unwrap();
        repo.insert(newer).await.unwrap();

        let logs = repo.list_by_vehicle(vehicle_id, user_id).await.unwrap();
        assert_eq!(logs.len(), 2);
        // Most recent date first
        assert_eq!(logs[0].station.as_deref(), Some("Shell"));
        assert_eq!(logs[1].station.as_deref(), Some("Pertamina"));
    }
}
