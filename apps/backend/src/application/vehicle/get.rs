use std::sync::Arc;

use uuid::Uuid;

use crate::{
    application::errors::AppError, domain::vehicle::entity::Vehicle,
    ports::repositories::VehicleRepository,
};

pub struct GetVehicleUseCase {
    pub repo: Arc<dyn VehicleRepository>,
}

impl GetVehicleUseCase {
    pub async fn execute(&self, id: Uuid, user_id: Uuid) -> Result<Vehicle, AppError> {
        self.repo
            .find_by_id(id, user_id)
            .await
            .map_err(AppError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    use async_trait::async_trait;
    use chrono::Utc;

    struct FakeVehicleRepo {
        vehicle: Option<Vehicle>,
    }

    #[async_trait]
    impl VehicleRepository for FakeVehicleRepo {
        async fn list_by_user(&self, _user_id: Uuid) -> RepositoryResult<Vec<Vehicle>> {
            Ok(vec![])
        }

        async fn find_by_id(&self, id: Uuid, user_id: Uuid) -> RepositoryResult<Vehicle> {
            match &self.vehicle {
                Some(v) if v.id == id && v.user_id == user_id => Ok(v.clone()),
                _ => Err(RepositoryError::NotFound),
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

    #[tokio::test]
    async fn get_own_vehicle_succeeds() {
        let user_id = Uuid::new_v4();
        let vehicle_id = Uuid::new_v4();
        let vehicle = Vehicle {
            id: vehicle_id,
            user_id,
            brand: "Honda".into(),
            model: "Brio".into(),
            year: 2021,
            plate_number: PlateNumber::new("B 9999 AAA".to_string()).unwrap(),
            color: None,
            fuel_type: FuelType::Petrol,
            current_odometer: Odometer::new(0),
            notes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let repo = Arc::new(FakeVehicleRepo {
            vehicle: Some(vehicle),
        });
        let uc = GetVehicleUseCase { repo };

        let result = uc.execute(vehicle_id, user_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, vehicle_id);
    }

    #[tokio::test]
    async fn get_other_user_vehicle_returns_not_found() {
        let owner_id = Uuid::new_v4();
        let intruder_id = Uuid::new_v4();
        let vehicle_id = Uuid::new_v4();
        let vehicle = Vehicle {
            id: vehicle_id,
            user_id: owner_id,
            brand: "Honda".into(),
            model: "Brio".into(),
            year: 2021,
            plate_number: PlateNumber::new("B 9999 AAA".to_string()).unwrap(),
            color: None,
            fuel_type: FuelType::Petrol,
            current_odometer: Odometer::new(0),
            notes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let repo = Arc::new(FakeVehicleRepo {
            vehicle: Some(vehicle),
        });
        let uc = GetVehicleUseCase { repo };

        let result = uc.execute(vehicle_id, intruder_id).await;
        assert!(matches!(result, Err(AppError::NotFound)));
    }
}
