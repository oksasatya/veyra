use std::sync::Arc;

use uuid::Uuid;

use crate::{
    application::errors::AppError, domain::vehicle::entity::Vehicle,
    ports::repositories::VehicleRepository,
};

pub struct ListVehiclesUseCase {
    pub repo: Arc<dyn VehicleRepository>,
}

impl ListVehiclesUseCase {
    pub async fn execute(&self, user_id: Uuid) -> Result<Vec<Vehicle>, AppError> {
        self.repo
            .list_by_user(user_id)
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
        vehicles: Vec<Vehicle>,
    }

    #[async_trait]
    impl VehicleRepository for FakeVehicleRepo {
        async fn list_by_user(&self, user_id: Uuid) -> RepositoryResult<Vec<Vehicle>> {
            Ok(self
                .vehicles
                .iter()
                .filter(|v| v.user_id == user_id)
                .cloned()
                .collect())
        }

        async fn find_by_id(&self, _id: Uuid, _user_id: Uuid) -> RepositoryResult<Vehicle> {
            Err(RepositoryError::NotFound)
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

    fn make_vehicle(user_id: Uuid, plate: &str) -> Vehicle {
        Vehicle {
            id: Uuid::new_v4(),
            user_id,
            brand: "Toyota".into(),
            model: "Avanza".into(),
            year: 2020,
            plate_number: PlateNumber::new(plate.to_string()).unwrap(),
            color: None,
            fuel_type: FuelType::Petrol,
            current_odometer: Odometer::new(0),
            notes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn list_returns_only_own_vehicles() {
        let user_a = Uuid::new_v4();
        let user_b = Uuid::new_v4();
        let repo = Arc::new(FakeVehicleRepo {
            vehicles: vec![
                make_vehicle(user_a, "B 0001 AAA"),
                make_vehicle(user_b, "B 0002 BBB"),
            ],
        });
        let uc = ListVehiclesUseCase { repo };

        let result = uc.execute(user_a).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].user_id, user_a);
    }

    #[tokio::test]
    async fn list_returns_empty_when_no_vehicles() {
        let user = Uuid::new_v4();
        let repo = Arc::new(FakeVehicleRepo { vehicles: vec![] });
        let uc = ListVehiclesUseCase { repo };

        let result = uc.execute(user).await.unwrap();
        assert!(result.is_empty());
    }
}
