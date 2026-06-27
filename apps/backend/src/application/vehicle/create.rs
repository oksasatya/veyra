use std::sync::Arc;

use uuid::Uuid;

use crate::{
    application::errors::AppError,
    domain::vehicle::entity::Vehicle,
    domain::vehicle::value_objects::{FuelType, PlateNumber},
    ports::repositories::{CreateVehicleParams, VehicleRepository},
};

pub struct CreateVehicleUseCase {
    pub repo: Arc<dyn VehicleRepository>,
}

pub struct CreateVehicleInput {
    pub user_id: Uuid,
    pub brand: String,
    pub model: String,
    pub year: i16,
    pub plate_number: String,
    pub color: Option<String>,
    pub fuel_type: String,
    pub current_odometer: u32,
    pub notes: Option<String>,
}

impl CreateVehicleUseCase {
    pub async fn execute(&self, input: CreateVehicleInput) -> Result<Vehicle, AppError> {
        // Validate value objects — domain errors surface as Validation
        PlateNumber::new(input.plate_number.clone()).map_err(AppError::from)?;
        FuelType::parse(&input.fuel_type)
            .map_err(|_| AppError::Validation(format!("invalid fuel_type: {}", input.fuel_type)))?;

        let params = CreateVehicleParams {
            user_id: input.user_id,
            brand: input.brand,
            model: input.model,
            year: input.year,
            plate_number: input.plate_number,
            color: input.color,
            fuel_type: input.fuel_type,
            current_odometer: input.current_odometer,
            notes: input.notes,
        };

        self.repo.insert(params).await.map_err(AppError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::vehicle::value_objects::{FuelType, Odometer, PlateNumber};
    use crate::{
        domain::vehicle::entity::Vehicle,
        ports::repositories::{CreateVehicleParams, RepositoryError, RepositoryResult},
    };
    use async_trait::async_trait;
    use chrono::Utc;

    struct FakeVehicleRepo {
        fail_with: Option<RepositoryError>,
    }

    #[async_trait]
    impl VehicleRepository for FakeVehicleRepo {
        async fn list_by_user(&self, _user_id: Uuid) -> RepositoryResult<Vec<Vehicle>> {
            Ok(vec![])
        }

        async fn find_by_id(&self, _id: Uuid, _user_id: Uuid) -> RepositoryResult<Vehicle> {
            Err(RepositoryError::NotFound)
        }

        async fn insert(&self, params: CreateVehicleParams) -> RepositoryResult<Vehicle> {
            if let Some(ref e) = self.fail_with {
                return Err(match e {
                    RepositoryError::NotFound => RepositoryError::NotFound,
                    RepositoryError::Conflict(m) => RepositoryError::Conflict(m.clone()),
                    RepositoryError::Database(m) => RepositoryError::Database(m.clone()),
                });
            }
            Ok(Vehicle {
                id: Uuid::new_v4(),
                user_id: params.user_id,
                brand: params.brand,
                model: params.model,
                year: params.year,
                plate_number: PlateNumber::new(params.plate_number).unwrap(),
                color: params.color,
                fuel_type: FuelType::parse(&params.fuel_type).unwrap(),
                current_odometer: Odometer::new(params.current_odometer),
                notes: params.notes,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }

        async fn update(
            &self,
            _id: Uuid,
            _user_id: Uuid,
            _params: crate::ports::repositories::UpdateVehicleParams,
        ) -> RepositoryResult<Vehicle> {
            Err(RepositoryError::NotFound)
        }

        async fn delete(&self, _id: Uuid, _user_id: Uuid) -> RepositoryResult<()> {
            Err(RepositoryError::NotFound)
        }
    }

    #[tokio::test]
    async fn valid_input_creates_vehicle() {
        let repo = Arc::new(FakeVehicleRepo { fail_with: None });
        let uc = CreateVehicleUseCase { repo };
        let result = uc
            .execute(CreateVehicleInput {
                user_id: Uuid::new_v4(),
                brand: "Toyota".into(),
                model: "Avanza".into(),
                year: 2020,
                plate_number: "B 1234 XYZ".into(),
                color: None,
                fuel_type: "petrol".into(),
                current_odometer: 0,
                notes: None,
            })
            .await;
        assert!(result.is_ok());
        let v = result.unwrap();
        assert_eq!(v.brand, "Toyota");
    }

    #[tokio::test]
    async fn empty_plate_number_returns_validation_error() {
        let repo = Arc::new(FakeVehicleRepo { fail_with: None });
        let uc = CreateVehicleUseCase { repo };
        let result = uc
            .execute(CreateVehicleInput {
                user_id: Uuid::new_v4(),
                brand: "Toyota".into(),
                model: "Avanza".into(),
                year: 2020,
                plate_number: "  ".into(),
                color: None,
                fuel_type: "petrol".into(),
                current_odometer: 0,
                notes: None,
            })
            .await;
        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[tokio::test]
    async fn invalid_fuel_type_returns_validation_error() {
        let repo = Arc::new(FakeVehicleRepo { fail_with: None });
        let uc = CreateVehicleUseCase { repo };
        let result = uc
            .execute(CreateVehicleInput {
                user_id: Uuid::new_v4(),
                brand: "Toyota".into(),
                model: "Avanza".into(),
                year: 2020,
                plate_number: "B 1234 XYZ".into(),
                color: None,
                fuel_type: "gasoline".into(),
                current_odometer: 0,
                notes: None,
            })
            .await;
        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[tokio::test]
    async fn repo_conflict_maps_to_app_conflict() {
        let repo = Arc::new(FakeVehicleRepo {
            fail_with: Some(RepositoryError::Conflict(
                "plate number already registered".into(),
            )),
        });
        let uc = CreateVehicleUseCase { repo };
        let result = uc
            .execute(CreateVehicleInput {
                user_id: Uuid::new_v4(),
                brand: "Toyota".into(),
                model: "Avanza".into(),
                year: 2020,
                plate_number: "B 1234 XYZ".into(),
                color: None,
                fuel_type: "petrol".into(),
                current_odometer: 0,
                notes: None,
            })
            .await;
        assert!(matches!(result, Err(AppError::Conflict(_))));
    }
}
