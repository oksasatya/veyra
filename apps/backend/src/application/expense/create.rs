use std::sync::Arc;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::{
    application::errors::AppError,
    domain::error_code::ErrorCode,
    domain::expense::entity::{Expense, ExpenseCategory},
    ports::repositories::{CreateExpenseParams, ExpenseRepository, VehicleRepository},
};

pub struct CreateExpenseUseCase {
    pub repo: Arc<dyn ExpenseRepository>,
    pub vehicle_repo: Arc<dyn VehicleRepository>,
}

pub struct CreateExpenseInput {
    pub vehicle_id: Uuid,
    pub user_id: Uuid,
    pub expense_date: NaiveDate,
    pub category: String,
    pub description: String,
    pub amount: Decimal,
}

impl CreateExpenseUseCase {
    /// Verifies that `vehicle_id` belongs to `user_id` before inserting the
    /// expense. Validates `category` against `ExpenseCategory::parse`.
    /// Returns `AppError::NotFound` if the vehicle is not owned by the caller,
    /// or `AppError::Validation` for an unknown category.
    pub async fn execute(&self, input: CreateExpenseInput) -> Result<Expense, AppError> {
        // Validate category before hitting the database
        let _ = ExpenseCategory::parse(&input.category).ok_or_else(|| {
            AppError::validation(
                ErrorCode::InvalidCategory,
                format!("unknown category: {}", input.category),
            )
        })?;

        // Ownership guard: vehicle must belong to the caller
        self.vehicle_repo
            .find_by_id(input.vehicle_id, input.user_id)
            .await
            .map_err(AppError::from)?;

        self.repo
            .insert(CreateExpenseParams {
                vehicle_id: input.vehicle_id,
                expense_date: input.expense_date,
                category: input.category,
                description: input.description,
                amount: input.amount,
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
            expense::entity::Expense,
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

    struct FakeExpenseRepo {
        fail_with: Option<RepositoryError>,
    }

    #[async_trait]
    impl ExpenseRepository for FakeExpenseRepo {
        async fn list_by_vehicle(
            &self,
            _vehicle_id: Uuid,
            _user_id: Uuid,
        ) -> RepositoryResult<Vec<Expense>> {
            Ok(vec![])
        }

        async fn insert(&self, params: CreateExpenseParams) -> RepositoryResult<Expense> {
            if let Some(ref e) = self.fail_with {
                return Err(match e {
                    RepositoryError::NotFound => RepositoryError::NotFound,
                    RepositoryError::Conflict(m) => RepositoryError::Conflict(m.clone()),
                    RepositoryError::Database(m) => RepositoryError::Database(m.clone()),
                });
            }
            Ok(Expense {
                id: Uuid::new_v4(),
                vehicle_id: params.vehicle_id,
                expense_date: params.expense_date,
                category: ExpenseCategory::parse(&params.category).unwrap(),
                description: params.description,
                amount: params.amount,
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

    fn make_input(vehicle_id: Uuid, user_id: Uuid) -> CreateExpenseInput {
        CreateExpenseInput {
            vehicle_id,
            user_id,
            expense_date: "2026-06-15".parse().unwrap(),
            category: "tire".into(),
            description: "Front tire replacement".into(),
            amount: Decimal::new(35_000_000, 2), // 350000.00
        }
    }

    #[tokio::test]
    async fn valid_input_creates_expense() {
        let vehicle_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let uc = CreateExpenseUseCase {
            repo: Arc::new(FakeExpenseRepo { fail_with: None }),
            vehicle_repo: Arc::new(FakeVehicleRepo {
                owner_id: user_id,
                vehicle_id,
            }),
        };

        let result = uc.execute(make_input(vehicle_id, user_id)).await;
        assert!(result.is_ok());
        let expense = result.unwrap();
        assert_eq!(expense.category, ExpenseCategory::Tire);
    }

    #[tokio::test]
    async fn invalid_category_returns_validation_error() {
        let vehicle_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let uc = CreateExpenseUseCase {
            repo: Arc::new(FakeExpenseRepo { fail_with: None }),
            vehicle_repo: Arc::new(FakeVehicleRepo {
                owner_id: user_id,
                vehicle_id,
            }),
        };

        let mut input = make_input(vehicle_id, user_id);
        input.category = "unknown_cat".into();

        let result = uc.execute(input).await;
        assert!(matches!(result, Err(AppError::Validation { .. })));
    }

    #[tokio::test]
    async fn wrong_owner_returns_not_found() {
        let vehicle_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let intruder_id = Uuid::new_v4();

        let uc = CreateExpenseUseCase {
            repo: Arc::new(FakeExpenseRepo { fail_with: None }),
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

        let uc = CreateExpenseUseCase {
            repo: Arc::new(FakeExpenseRepo {
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
