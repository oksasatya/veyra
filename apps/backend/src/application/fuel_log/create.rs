use std::sync::Arc;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::{
    application::errors::AppError,
    domain::fuel_log::entity::FuelLog,
    ports::repositories::{CreateFuelLogParams, FuelLogRepository, VehicleRepository},
};

pub struct CreateFuelLogUseCase {
    pub repo: Arc<dyn FuelLogRepository>,
    pub vehicle_repo: Arc<dyn VehicleRepository>,
}

pub struct CreateFuelLogInput {
    pub vehicle_id: Uuid,
    pub user_id: Uuid,
    pub log_date: NaiveDate,
    pub odometer: u32,
    pub liters: Decimal,
    pub price_per_liter: Decimal,
    pub station: Option<String>,
    pub is_full_tank: bool,
}

impl CreateFuelLogUseCase {
    /// Verifies that `vehicle_id` belongs to `user_id` before inserting the
    /// fuel log. Returns `AppError::NotFound` if the vehicle is not owned by
    /// the caller (ownership guard — no cross-user writes).
    pub async fn execute(&self, input: CreateFuelLogInput) -> Result<FuelLog, AppError> {
        // Ownership guard: vehicle must belong to the caller
        self.vehicle_repo
            .find_by_id(input.vehicle_id, input.user_id)
            .await
            .map_err(AppError::from)?;

        self.repo
            .insert(CreateFuelLogParams {
                vehicle_id: input.vehicle_id,
                log_date: input.log_date,
                odometer: input.odometer,
                liters: input.liters,
                price_per_liter: input.price_per_liter,
                station: input.station,
                is_full_tank: input.is_full_tank,
            })
            .await
            .map_err(AppError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::{
            fuel_log::entity::FuelLog,
            vehicle::{
                entity::Vehicle,
                value_objects::{FuelType, Odometer, PlateNumber},
            },
        },
        ports::repositories::{
            CreateFuelLogParams, CreateVehicleParams, RepositoryError, RepositoryResult,
            UpdateVehicleParams,
        },
    };
    use async_trait::async_trait;
    use chrono::Utc;

    struct FakeVehicleRepo {
        owner_id: Uuid,
        vehicle_id: Uuid,
    }

    #[async_trait]
    impl VehicleRepository for FakeVehicleRepo {
        async fn list_by_user(&self, _user_id: Uuid) -> RepositoryResult<Vec<Vehicle>> {
            Ok(vec![])
        }

        async fn find_by_id(&self, id: Uuid, user_id: Uuid) -> RepositoryResult<Vehicle> {
            if id == self.vehicle_id && user_id == self.owner_id {
                Ok(fake_vehicle(self.vehicle_id, self.owner_id))
            } else {
                Err(RepositoryError::NotFound)
            }
        }

        async fn insert(&self, _params: CreateVehicleParams) -> RepositoryResult<Vehicle> {
            Err(RepositoryError::NotFound)
        }

        async fn update(
            &self,
            _id: Uuid,
            _user_id: Uuid,
            _params: UpdateVehicleParams,
        ) -> RepositoryResult<Vehicle> {
            Err(RepositoryError::NotFound)
        }

        async fn delete(&self, _id: Uuid, _user_id: Uuid) -> RepositoryResult<()> {
            Err(RepositoryError::NotFound)
        }
    }

    struct FakeFuelLogRepo {
        fail_with: Option<RepositoryError>,
    }

    #[async_trait]
    impl FuelLogRepository for FakeFuelLogRepo {
        async fn list_by_vehicle(
            &self,
            _vehicle_id: Uuid,
            _user_id: Uuid,
        ) -> RepositoryResult<Vec<FuelLog>> {
            Ok(vec![])
        }

        async fn insert(&self, params: CreateFuelLogParams) -> RepositoryResult<FuelLog> {
            if let Some(ref e) = self.fail_with {
                return Err(match e {
                    RepositoryError::NotFound => RepositoryError::NotFound,
                    RepositoryError::Conflict(m) => RepositoryError::Conflict(m.clone()),
                    RepositoryError::Database(m) => RepositoryError::Database(m.clone()),
                });
            }
            let liters = params.liters;
            let price_per_liter = params.price_per_liter;
            Ok(FuelLog {
                id: Uuid::new_v4(),
                vehicle_id: params.vehicle_id,
                log_date: params.log_date,
                odometer: params.odometer,
                liters,
                price_per_liter,
                total_cost: liters * price_per_liter,
                station: params.station,
                is_full_tank: params.is_full_tank,
                created_at: Utc::now(),
            })
        }
    }

    fn fake_vehicle(id: Uuid, user_id: Uuid) -> Vehicle {
        Vehicle {
            id,
            user_id,
            brand: "Toyota".into(),
            model: "Avanza".into(),
            year: 2020,
            plate_number: PlateNumber::new("B 1234 XYZ".into()).unwrap(),
            color: None,
            fuel_type: FuelType::parse("petrol").unwrap(),
            current_odometer: Odometer::new(0),
            notes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_input(vehicle_id: Uuid, user_id: Uuid) -> CreateFuelLogInput {
        CreateFuelLogInput {
            vehicle_id,
            user_id,
            log_date: "2026-01-20".parse().unwrap(),
            odometer: 10_000,
            liters: Decimal::new(400, 1),             // 40.0
            price_per_liter: Decimal::new(100000, 1), // 10000.0
            station: Some("Shell".into()),
            is_full_tank: true,
        }
    }

    #[tokio::test]
    async fn valid_input_creates_fuel_log() {
        let vehicle_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let uc = CreateFuelLogUseCase {
            repo: Arc::new(FakeFuelLogRepo { fail_with: None }),
            vehicle_repo: Arc::new(FakeVehicleRepo {
                owner_id: user_id,
                vehicle_id,
            }),
        };

        let result = uc.execute(make_input(vehicle_id, user_id)).await;
        assert!(result.is_ok());
        let log = result.unwrap();
        assert_eq!(log.odometer, 10_000);
        assert!(log.is_full_tank);
        // total_cost = 40.0 * 10000.0 = 400000
        assert_eq!(log.total_cost, Decimal::new(4_000_000, 1));
    }

    #[tokio::test]
    async fn wrong_owner_returns_not_found() {
        let vehicle_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let intruder_id = Uuid::new_v4();

        let uc = CreateFuelLogUseCase {
            repo: Arc::new(FakeFuelLogRepo { fail_with: None }),
            vehicle_repo: Arc::new(FakeVehicleRepo {
                owner_id,
                vehicle_id,
            }),
        };

        let result = uc.execute(make_input(vehicle_id, intruder_id)).await;
        assert!(matches!(result, Err(AppError::NotFound)));
    }

    #[tokio::test]
    async fn repo_database_error_maps_to_internal() {
        let vehicle_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let uc = CreateFuelLogUseCase {
            repo: Arc::new(FakeFuelLogRepo {
                fail_with: Some(RepositoryError::Database("db error".into())),
            }),
            vehicle_repo: Arc::new(FakeVehicleRepo {
                owner_id: user_id,
                vehicle_id,
            }),
        };

        let result = uc.execute(make_input(vehicle_id, user_id)).await;
        assert!(matches!(result, Err(AppError::Internal(_))));
    }
}
