use std::sync::Arc;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::{
    application::errors::AppError,
    domain::service_record::entity::ServiceRecord,
    ports::repositories::{CreateServiceRecordParams, ServiceRecordRepository, VehicleRepository},
};

pub struct CreateServiceRecordUseCase {
    pub repo: Arc<dyn ServiceRecordRepository>,
    pub vehicle_repo: Arc<dyn VehicleRepository>,
}

pub struct CreateServiceRecordInput {
    pub vehicle_id: Uuid,
    pub user_id: Uuid,
    pub service_date: NaiveDate,
    pub odometer: u32,
    pub description: String,
    pub workshop: Option<String>,
    pub cost: Option<Decimal>,
    pub notes: Option<String>,
}

impl CreateServiceRecordUseCase {
    /// Verifies that `vehicle_id` belongs to `user_id` before inserting the
    /// service record. Returns `AppError::NotFound` if the vehicle is not owned
    /// by the caller (ownership guard — no cross-user writes).
    pub async fn execute(
        &self,
        input: CreateServiceRecordInput,
    ) -> Result<ServiceRecord, AppError> {
        // Ownership guard: vehicle must belong to the caller
        self.vehicle_repo
            .find_by_id(input.vehicle_id, input.user_id)
            .await
            .map_err(AppError::from)?;

        self.repo
            .insert(CreateServiceRecordParams {
                vehicle_id: input.vehicle_id,
                service_date: input.service_date,
                odometer: input.odometer,
                description: input.description,
                workshop: input.workshop,
                cost: input.cost,
                notes: input.notes,
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
            service_record::entity::ServiceRecord,
            vehicle::{
                entity::Vehicle,
                value_objects::{FuelType, Odometer, PlateNumber},
            },
        },
        ports::repositories::{
            CreateServiceRecordParams, CreateVehicleParams, RepositoryError, RepositoryResult,
            UpdateVehicleParams,
        },
    };
    use async_trait::async_trait;
    use chrono::Utc;
    use rust_decimal::Decimal;

    // ── Fake repos ────────────────────────────────────────────────────────────

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

    struct FakeServiceRecordRepo {
        fail_with: Option<RepositoryError>,
    }

    #[async_trait]
    impl ServiceRecordRepository for FakeServiceRecordRepo {
        async fn list_by_vehicle(
            &self,
            _vehicle_id: Uuid,
            _user_id: Uuid,
        ) -> RepositoryResult<Vec<ServiceRecord>> {
            Ok(vec![])
        }

        async fn insert(
            &self,
            params: CreateServiceRecordParams,
        ) -> RepositoryResult<ServiceRecord> {
            if let Some(ref e) = self.fail_with {
                return Err(match e {
                    RepositoryError::NotFound => RepositoryError::NotFound,
                    RepositoryError::Conflict(m) => RepositoryError::Conflict(m.clone()),
                    RepositoryError::Database(m) => RepositoryError::Database(m.clone()),
                });
            }
            Ok(ServiceRecord {
                id: Uuid::new_v4(),
                vehicle_id: params.vehicle_id,
                service_date: params.service_date,
                odometer: params.odometer,
                description: params.description,
                workshop: params.workshop,
                cost: params.cost,
                notes: params.notes,
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

    fn make_input(vehicle_id: Uuid, user_id: Uuid) -> CreateServiceRecordInput {
        CreateServiceRecordInput {
            vehicle_id,
            user_id,
            service_date: "2026-01-15".parse().unwrap(),
            odometer: 5_000,
            description: "Oil change".into(),
            workshop: Some("Fast Lube".into()),
            cost: Some(Decimal::new(15000, 2)),
            notes: None,
        }
    }

    // ── Tests ─────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn valid_input_creates_service_record() {
        let vehicle_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let uc = CreateServiceRecordUseCase {
            repo: Arc::new(FakeServiceRecordRepo { fail_with: None }),
            vehicle_repo: Arc::new(FakeVehicleRepo {
                owner_id: user_id,
                vehicle_id,
            }),
        };

        let result = uc.execute(make_input(vehicle_id, user_id)).await;
        assert!(result.is_ok());
        let rec = result.unwrap();
        assert_eq!(rec.description, "Oil change");
        assert_eq!(rec.odometer, 5_000);
    }

    #[tokio::test]
    async fn wrong_owner_returns_not_found() {
        let vehicle_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let intruder_id = Uuid::new_v4();

        let uc = CreateServiceRecordUseCase {
            repo: Arc::new(FakeServiceRecordRepo { fail_with: None }),
            vehicle_repo: Arc::new(FakeVehicleRepo {
                owner_id,
                vehicle_id,
            }),
        };

        let result = uc
            .execute(make_input(vehicle_id, intruder_id))
            .await;
        assert!(matches!(result, Err(AppError::NotFound)));
    }

    #[tokio::test]
    async fn repo_database_error_maps_to_internal() {
        let vehicle_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let uc = CreateServiceRecordUseCase {
            repo: Arc::new(FakeServiceRecordRepo {
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
