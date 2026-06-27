use std::sync::Arc;

use uuid::Uuid;

use crate::{
    application::errors::AppError,
    domain::expense::entity::Expense,
    ports::repositories::{ExpenseRepository, VehicleRepository},
};

pub struct ListExpensesUseCase {
    pub repo: Arc<dyn ExpenseRepository>,
    pub vehicle_repo: Arc<dyn VehicleRepository>,
}

impl ListExpensesUseCase {
    /// Verifies ownership of `vehicle_id` before listing its expenses.
    /// Returns `AppError::NotFound` when the vehicle is not owned by `user_id`.
    pub async fn execute(&self, vehicle_id: Uuid, user_id: Uuid) -> Result<Vec<Expense>, AppError> {
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
            expense::entity::{Expense, ExpenseCategory},
            vehicle::{
                entity::Vehicle,
                value_objects::{FuelType, Odometer, PlateNumber},
            },
        },
        ports::repositories::{
            CreateExpenseParams, CreateVehicleParams, RepositoryError, RepositoryResult,
            UpdateVehicleParams,
        },
    };
    use async_trait::async_trait;
    use chrono::Utc;
    use rust_decimal::Decimal;

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

    struct FakeExpenseRepo {
        expenses: Vec<Expense>,
    }

    #[async_trait]
    impl ExpenseRepository for FakeExpenseRepo {
        async fn list_by_vehicle(
            &self,
            _vehicle_id: Uuid,
            _user_id: Uuid,
        ) -> RepositoryResult<Vec<Expense>> {
            Ok(self.expenses.clone())
        }

        async fn insert(&self, _params: CreateExpenseParams) -> RepositoryResult<Expense> {
            Err(RepositoryError::NotFound)
        }
    }

    #[tokio::test]
    async fn returns_expenses_for_owned_vehicle() {
        let vehicle_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let expense = Expense {
            id: Uuid::new_v4(),
            vehicle_id,
            expense_date: "2026-06-15".parse().unwrap(),
            category: ExpenseCategory::Tire,
            description: "Front tire".into(),
            amount: Decimal::new(35_000_000, 2),
            created_at: Utc::now(),
        };

        let uc = ListExpensesUseCase {
            repo: Arc::new(FakeExpenseRepo {
                expenses: vec![expense],
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

        let uc = ListExpensesUseCase {
            repo: Arc::new(FakeExpenseRepo { expenses: vec![] }),
            vehicle_repo: Arc::new(FakeVehicleRepo {
                owner_id,
                vehicle_id,
            }),
        };

        let result = uc.execute(vehicle_id, intruder_id).await;
        assert!(matches!(result, Err(AppError::NotFound)));
    }
}
