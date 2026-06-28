use std::sync::Arc;

use uuid::Uuid;

use crate::{
    application::errors::AppError,
    domain::error_code::ErrorCode,
    domain::vehicle::entity::Vehicle,
    domain::vehicle::value_objects::FuelType,
    ports::repositories::{UpdateVehicleParams, VehicleRepository},
};

pub struct UpdateVehicleUseCase {
    pub repo: Arc<dyn VehicleRepository>,
}

pub struct UpdateVehicleInput {
    pub brand: String,
    pub model: String,
    pub year: i16,
    pub color: Option<String>,
    pub fuel_type: String,
    pub current_odometer: u32,
    pub notes: Option<String>,
}

impl UpdateVehicleUseCase {
    pub async fn execute(
        &self,
        id: Uuid,
        user_id: Uuid,
        input: UpdateVehicleInput,
    ) -> Result<Vehicle, AppError> {
        // Validate fuel_type before hitting the repo — mirrors CreateVehicleUseCase
        FuelType::parse(&input.fuel_type).map_err(|_| {
            AppError::validation(
                ErrorCode::InvalidFuelType,
                format!("invalid fuel_type: {}", input.fuel_type),
            )
        })?;

        let params = UpdateVehicleParams {
            brand: input.brand,
            model: input.model,
            year: input.year,
            color: input.color,
            fuel_type: input.fuel_type,
            current_odometer: input.current_odometer,
            notes: input.notes,
        };
        self.repo
            .update(id, user_id, params)
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

        async fn find_by_id(&self, _id: Uuid, _user_id: Uuid) -> RepositoryResult<Vehicle> {
            Err(RepositoryError::NotFound)
        }

        async fn insert(&self, _params: CreateVehicleParams) -> RepositoryResult<Vehicle> {
            Err(RepositoryError::NotFound)
        }

        async fn update(
            &self,
            id: Uuid,
            user_id: Uuid,
            params: UpdateVehicleParams,
        ) -> RepositoryResult<Vehicle> {
            match &self.vehicle {
                Some(v) if v.id == id && v.user_id == user_id => Ok(Vehicle {
                    brand: params.brand,
                    model: params.model,
                    year: params.year,
                    color: params.color,
                    fuel_type: FuelType::parse(&params.fuel_type)
                        .map_err(|e| RepositoryError::Database(e.to_string()))?,
                    current_odometer: Odometer::new(params.current_odometer),
                    notes: params.notes,
                    ..v.clone()
                }),
                _ => Err(RepositoryError::NotFound),
            }
        }

        async fn delete(&self, _id: Uuid, _user_id: Uuid) -> RepositoryResult<()> {
            Err(RepositoryError::NotFound)
        }
    }

    #[tokio::test]
    async fn update_own_vehicle_succeeds() {
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
        let uc = UpdateVehicleUseCase { repo };

        let result = uc
            .execute(
                vehicle_id,
                user_id,
                UpdateVehicleInput {
                    brand: "Toyota".into(),
                    model: "Avanza".into(),
                    year: 2022,
                    color: Some("Red".into()),
                    fuel_type: "diesel".into(),
                    current_odometer: 10_000,
                    notes: None,
                },
            )
            .await;

        assert!(result.is_ok());
        let v = result.unwrap();
        assert_eq!(v.brand, "Toyota");
        assert_eq!(v.current_odometer.value(), 10_000);
    }

    #[tokio::test]
    async fn update_other_user_vehicle_returns_not_found() {
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
        let uc = UpdateVehicleUseCase { repo };

        let result = uc
            .execute(
                vehicle_id,
                intruder_id,
                UpdateVehicleInput {
                    brand: "Toyota".into(),
                    model: "Avanza".into(),
                    year: 2022,
                    color: None,
                    fuel_type: "petrol".into(),
                    current_odometer: 0,
                    notes: None,
                },
            )
            .await;

        assert!(matches!(result, Err(AppError::NotFound)));
    }

    #[tokio::test]
    async fn update_with_invalid_fuel_type_returns_error() {
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
        let uc = UpdateVehicleUseCase { repo };

        let result = uc
            .execute(
                vehicle_id,
                user_id,
                UpdateVehicleInput {
                    brand: "Honda".into(),
                    model: "Brio".into(),
                    year: 2021,
                    color: None,
                    fuel_type: "ROCKET_FUEL".into(),
                    current_odometer: 0,
                    notes: None,
                },
            )
            .await;

        assert!(matches!(result, Err(AppError::Validation { .. })));
    }
}
