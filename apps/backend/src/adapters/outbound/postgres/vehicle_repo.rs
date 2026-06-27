use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::vehicle::{
        entity::Vehicle,
        value_objects::{FuelType, Odometer, PlateNumber},
    },
    ports::repositories::{
        CreateVehicleParams, RepositoryError, RepositoryResult, UpdateVehicleParams,
        VehicleRepository,
    },
};

use super::models::VehicleRow;

/// Unique constraint name on (user_id, plate_number) in the vehicles table.
const PLATE_CONSTRAINT: &str = "vehicles_user_id_plate_number_key";

/// Postgres implementation of [`VehicleRepository`].
pub struct PgVehicleRepo {
    pool: PgPool,
}

impl PgVehicleRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Maps a `sqlx::Error` to `RepositoryError`, detecting the plate-number
/// uniqueness constraint so callers get `Conflict` rather than a generic error.
fn map_pg_err(e: sqlx::Error) -> RepositoryError {
    if let sqlx::Error::Database(ref db_err) = e {
        if db_err.constraint() == Some(PLATE_CONSTRAINT) {
            return RepositoryError::Conflict(
                "plate number already registered for this account".into(),
            );
        }
    }
    RepositoryError::Database(e.to_string())
}

/// Converts a raw `VehicleRow` (sqlx) into a domain `Vehicle`.
///
/// # Errors
/// Returns `RepositoryError::Database` if stored values fail domain
/// validation — which indicates data corruption since validation is
/// enforced on write.
fn row_to_vehicle(row: VehicleRow) -> Result<Vehicle, RepositoryError> {
    let plate_number =
        PlateNumber::new(row.plate_number).map_err(|e| RepositoryError::Database(e.to_string()))?;
    let fuel_type =
        FuelType::parse(&row.fuel_type).map_err(|e| RepositoryError::Database(e.to_string()))?;

    Ok(Vehicle {
        id: row.id,
        user_id: row.user_id,
        brand: row.brand,
        model: row.model,
        year: row.year,
        plate_number,
        color: row.color,
        fuel_type,
        current_odometer: Odometer::new(row.current_odometer as u32),
        notes: row.notes,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

const SELECT_COLS: &str = "id, user_id, brand, model, year, plate_number, color, fuel_type, \
     current_odometer, notes, created_at, updated_at";

#[async_trait]
impl VehicleRepository for PgVehicleRepo {
    async fn list_by_user(&self, user_id: Uuid) -> RepositoryResult<Vec<Vehicle>> {
        let query = format!(
            "SELECT {SELECT_COLS} FROM vehicles WHERE user_id = $1 ORDER BY created_at DESC"
        );
        let rows = sqlx::query_as::<_, VehicleRow>(&query)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        rows.into_iter().map(row_to_vehicle).collect()
    }

    async fn find_by_id(&self, id: Uuid, user_id: Uuid) -> RepositoryResult<Vehicle> {
        let query = format!("SELECT {SELECT_COLS} FROM vehicles WHERE id = $1 AND user_id = $2");
        let row = sqlx::query_as::<_, VehicleRow>(&query)
            .bind(id)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?
            .ok_or(RepositoryError::NotFound)?;

        row_to_vehicle(row)
    }

    async fn insert(&self, params: CreateVehicleParams) -> RepositoryResult<Vehicle> {
        let query = format!(
            r#"INSERT INTO vehicles (user_id, brand, model, year, plate_number, color,
                                    fuel_type, current_odometer, notes)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
               RETURNING {SELECT_COLS}"#
        );
        let row = sqlx::query_as::<_, VehicleRow>(&query)
            .bind(params.user_id)
            .bind(&params.brand)
            .bind(&params.model)
            .bind(params.year)
            .bind(&params.plate_number)
            .bind(&params.color)
            .bind(&params.fuel_type)
            .bind(params.current_odometer as i32)
            .bind(&params.notes)
            .fetch_one(&self.pool)
            .await
            .map_err(map_pg_err)?;

        row_to_vehicle(row)
    }

    async fn update(
        &self,
        id: Uuid,
        user_id: Uuid,
        params: UpdateVehicleParams,
    ) -> RepositoryResult<Vehicle> {
        let query = format!(
            r#"UPDATE vehicles
               SET brand = $3, model = $4, year = $5, color = $6, fuel_type = $7,
                   current_odometer = $8, notes = $9, updated_at = NOW()
               WHERE id = $1 AND user_id = $2
               RETURNING {SELECT_COLS}"#
        );
        let row = sqlx::query_as::<_, VehicleRow>(&query)
            .bind(id)
            .bind(user_id)
            .bind(&params.brand)
            .bind(&params.model)
            .bind(params.year)
            .bind(&params.color)
            .bind(&params.fuel_type)
            .bind(params.current_odometer as i32)
            .bind(&params.notes)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?
            .ok_or(RepositoryError::NotFound)?;

        row_to_vehicle(row)
    }

    async fn delete(&self, id: Uuid, user_id: Uuid) -> RepositoryResult<()> {
        let result = sqlx::query("DELETE FROM vehicles WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use testcontainers_modules::postgres::Postgres;
    use testcontainers_modules::testcontainers::{runners::AsyncRunner, ImageExt};

    /// Spins up Postgres 16, runs all migrations, returns pool + container handle.
    /// Container must outlive the pool — callers bind it with `_container`.
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
        let url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");
        let pool = PgPool::connect(&url).await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        (pool, container)
    }

    /// Insert a user directly so we have a valid user_id foreign key.
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

    fn vehicle_params(user_id: Uuid, plate: &str) -> CreateVehicleParams {
        CreateVehicleParams {
            user_id,
            brand: "Toyota".into(),
            model: "Avanza".into(),
            year: 2020,
            plate_number: plate.to_string(),
            color: None,
            fuel_type: "petrol".into(),
            current_odometer: 0,
            notes: None,
        }
    }

    #[tokio::test]
    async fn insert_and_find_by_id() {
        let (pool, _container) = setup_pool().await;
        let user_id = insert_user(&pool).await;
        let repo = PgVehicleRepo::new(pool);

        let created = repo
            .insert(vehicle_params(user_id, "B 0001 AAA"))
            .await
            .unwrap();
        let found = repo.find_by_id(created.id, user_id).await.unwrap();

        assert_eq!(found.id, created.id);
        assert_eq!(found.brand, "Toyota");
        assert_eq!(found.plate_number.as_str(), "B 0001 AAA");
    }

    #[tokio::test]
    async fn list_by_user_returns_only_own_vehicles() {
        let (pool, _container) = setup_pool().await;
        let user_a = insert_user(&pool).await;
        let user_b = insert_user(&pool).await;
        let repo = PgVehicleRepo::new(pool);

        repo.insert(vehicle_params(user_a, "B 1111 AAA"))
            .await
            .unwrap();
        repo.insert(vehicle_params(user_b, "B 2222 BBB"))
            .await
            .unwrap();

        let list_a = repo.list_by_user(user_a).await.unwrap();
        let list_b = repo.list_by_user(user_b).await.unwrap();

        assert_eq!(list_a.len(), 1);
        assert_eq!(list_a[0].user_id, user_a);
        assert_eq!(list_b.len(), 1);
        assert_eq!(list_b[0].user_id, user_b);
    }

    #[tokio::test]
    async fn find_by_id_wrong_user_returns_not_found() {
        let (pool, _container) = setup_pool().await;
        let user_a = insert_user(&pool).await;
        let user_b = insert_user(&pool).await;
        let repo = PgVehicleRepo::new(pool);

        let created = repo
            .insert(vehicle_params(user_a, "B 3333 CCC"))
            .await
            .unwrap();
        let result = repo.find_by_id(created.id, user_b).await;

        assert!(matches!(result, Err(RepositoryError::NotFound)));
    }

    #[tokio::test]
    async fn update_vehicle_changes_fields() {
        let (pool, _container) = setup_pool().await;
        let user_id = insert_user(&pool).await;
        let repo = PgVehicleRepo::new(pool);

        let created = repo
            .insert(vehicle_params(user_id, "B 4444 DDD"))
            .await
            .unwrap();

        let updated = repo
            .update(
                created.id,
                user_id,
                UpdateVehicleParams {
                    brand: "Honda".into(),
                    model: "Jazz".into(),
                    year: 2023,
                    color: Some("Blue".into()),
                    fuel_type: "diesel".into(),
                    current_odometer: 15_000,
                    notes: Some("serviced".into()),
                },
            )
            .await
            .unwrap();

        assert_eq!(updated.brand, "Honda");
        assert_eq!(updated.current_odometer.value(), 15_000);
        assert_eq!(updated.color.as_deref(), Some("Blue"));
    }

    #[tokio::test]
    async fn update_wrong_user_returns_not_found() {
        let (pool, _container) = setup_pool().await;
        let user_a = insert_user(&pool).await;
        let user_b = insert_user(&pool).await;
        let repo = PgVehicleRepo::new(pool);

        let created = repo
            .insert(vehicle_params(user_a, "B 5555 EEE"))
            .await
            .unwrap();
        let result = repo
            .update(
                created.id,
                user_b,
                UpdateVehicleParams {
                    brand: "Nissan".into(),
                    model: "Livina".into(),
                    year: 2022,
                    color: None,
                    fuel_type: "petrol".into(),
                    current_odometer: 0,
                    notes: None,
                },
            )
            .await;

        assert!(matches!(result, Err(RepositoryError::NotFound)));
    }

    #[tokio::test]
    async fn delete_vehicle_removes_it() {
        let (pool, _container) = setup_pool().await;
        let user_id = insert_user(&pool).await;
        let repo = PgVehicleRepo::new(pool);

        let created = repo
            .insert(vehicle_params(user_id, "B 6666 FFF"))
            .await
            .unwrap();
        repo.delete(created.id, user_id).await.unwrap();

        let result = repo.find_by_id(created.id, user_id).await;
        assert!(matches!(result, Err(RepositoryError::NotFound)));
    }

    #[tokio::test]
    async fn delete_wrong_user_returns_not_found() {
        let (pool, _container) = setup_pool().await;
        let user_a = insert_user(&pool).await;
        let user_b = insert_user(&pool).await;
        let repo = PgVehicleRepo::new(pool);

        let created = repo
            .insert(vehicle_params(user_a, "B 7777 GGG"))
            .await
            .unwrap();
        let result = repo.delete(created.id, user_b).await;

        assert!(matches!(result, Err(RepositoryError::NotFound)));
    }

    #[tokio::test]
    async fn duplicate_plate_same_user_returns_conflict() {
        let (pool, _container) = setup_pool().await;
        let user_id = insert_user(&pool).await;
        let repo = PgVehicleRepo::new(pool);

        repo.insert(vehicle_params(user_id, "B 8888 HHH"))
            .await
            .unwrap();
        let result = repo.insert(vehicle_params(user_id, "B 8888 HHH")).await;

        assert!(matches!(result, Err(RepositoryError::Conflict(_))));
    }

    #[tokio::test]
    async fn same_plate_different_users_is_allowed() {
        let (pool, _container) = setup_pool().await;
        let user_a = insert_user(&pool).await;
        let user_b = insert_user(&pool).await;
        let repo = PgVehicleRepo::new(pool);

        repo.insert(vehicle_params(user_a, "B 9999 III"))
            .await
            .unwrap();
        let result = repo.insert(vehicle_params(user_b, "B 9999 III")).await;

        // Different users → no conflict
        assert!(result.is_ok());
    }
}
