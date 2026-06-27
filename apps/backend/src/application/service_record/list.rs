use std::sync::Arc;

use uuid::Uuid;

use crate::{
    application::errors::AppError,
    domain::service_record::entity::ServiceRecord,
    ports::repositories::{ServiceRecordRepository, VehicleRepository},
};

pub struct ListServiceRecordsUseCase {
    pub repo: Arc<dyn ServiceRecordRepository>,
    pub vehicle_repo: Arc<dyn VehicleRepository>,
}

impl ListServiceRecordsUseCase {
    /// Verifies ownership of `vehicle_id` before listing its service records.
    /// Returns `AppError::NotFound` when the vehicle is not owned by `user_id`.
    pub async fn execute(
        &self,
        vehicle_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<ServiceRecord>, AppError> {
        // Ownership guard: vehicle must belong to the caller
        self.vehicle_repo
            .find_by_id(vehicle_id, user_id)
            .await
            .map_err(AppError::from)?;

        self.repo
            .list_by_vehicle(vehicle_id, user_id)
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
                Ok(Vehicle {
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
                })
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
        records: Vec<ServiceRecord>,
    }

    #[async_trait]
    impl ServiceRecordRepository for FakeServiceRecordRepo {
        async fn list_by_vehicle(
            &self,
            _vehicle_id: Uuid,
            _user_id: Uuid,
        ) -> RepositoryResult<Vec<ServiceRecord>> {
            Ok(self.records.clone())
        }

        async fn insert(
            &self,
            _params: CreateServiceRecordParams,
        ) -> RepositoryResult<ServiceRecord> {
            Err(RepositoryError::NotFound)
        }
    }

    #[tokio::test]
    async fn returns_records_for_owned_vehicle() {
        let vehicle_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let rec = ServiceRecord {
            id: Uuid::new_v4(),
            vehicle_id,
            service_date: "2026-01-15".parse().unwrap(),
            odometer: 5_000,
            description: "Oil change".into(),
            workshop: None,
            cost: None,
            notes: None,
            created_at: Utc::now(),
        };

        let uc = ListServiceRecordsUseCase {
            repo: Arc::new(FakeServiceRecordRepo {
                records: vec![rec],
            }),
            vehicle_repo: Arc::new(FakeVehicleRepo {
                owner_id: user_id,
                vehicle_id,
            }),
        };

        let result = uc.execute(vehicle_id, user_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn wrong_owner_returns_not_found() {
        let vehicle_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let intruder_id = Uuid::new_v4();

        let uc = ListServiceRecordsUseCase {
            repo: Arc::new(FakeServiceRecordRepo { records: vec![] }),
            vehicle_repo: Arc::new(FakeVehicleRepo {
                owner_id,
                vehicle_id,
            }),
        };

        let result = uc.execute(vehicle_id, intruder_id).await;
        assert!(matches!(result, Err(AppError::NotFound)));
    }
}
