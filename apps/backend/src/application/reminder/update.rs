use std::sync::Arc;

use chrono::NaiveDate;
use uuid::Uuid;

use crate::{
    application::errors::AppError,
    domain::{
        errors::DomainError,
        reminder::entity::{Reminder, ReminderType},
    },
    ports::repositories::{ReminderRepository, UpdateReminderParams, VehicleRepository},
};

pub struct UpdateReminderUseCase {
    pub repo: Arc<dyn ReminderRepository>,
    pub vehicle_repo: Arc<dyn VehicleRepository>,
}

/// All fields are optional — only provided fields are applied.
pub struct PatchReminderInput {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub user_id: Uuid,
    pub is_completed: Option<bool>,
    pub due_date: Option<NaiveDate>,
    pub due_odometer: Option<u32>,
    pub notes: Option<String>,
}

impl UpdateReminderUseCase {
    /// Verifies vehicle ownership, fetches the existing reminder, applies the
    /// patch, re-validates that at least one due trigger remains set, then
    /// persists via `repo.update()`.
    pub async fn execute(&self, input: PatchReminderInput) -> Result<Reminder, AppError> {
        // Ownership guard: vehicle must belong to the caller
        self.vehicle_repo
            .find_by_id(input.vehicle_id, input.user_id)
            .await
            .map_err(AppError::from)?;

        // Fetch the existing reminder — returns 404 if not found or wrong user
        let existing = self
            .repo
            .find_by_id(input.id, input.vehicle_id, input.user_id)
            .await
            .map_err(AppError::from)?;

        // Apply patch: caller-supplied value wins; existing value is the fallback
        let due_date = merge_option(input.due_date, existing.due_date);
        let due_odometer = merge_option(input.due_odometer, existing.due_odometer);

        // Re-validate due triggers on the merged state
        validate_due_triggers(&existing.reminder_type, due_date, due_odometer)?;

        self.repo
            .update(
                input.id,
                input.vehicle_id,
                input.user_id,
                UpdateReminderParams {
                    is_completed: input.is_completed,
                    due_date,
                    due_odometer,
                    notes: input.notes,
                },
            )
            .await
            .map_err(AppError::from)
    }
}

/// Returns `new_val` if it is `Some`; otherwise falls back to `existing`.
fn merge_option<T>(new_val: Option<T>, existing: Option<T>) -> Option<T> {
    if new_val.is_some() {
        new_val
    } else {
        existing
    }
}

/// Validates that the required due trigger fields are present after the patch.
fn validate_due_triggers(
    reminder_type: &ReminderType,
    due_date: Option<NaiveDate>,
    due_odometer: Option<u32>,
) -> Result<(), AppError> {
    match reminder_type {
        ReminderType::Date | ReminderType::Both if due_date.is_none() => Err(AppError::Validation(
            DomainError::MissingDueDate.to_string(),
        )),
        ReminderType::Odometer | ReminderType::Both if due_odometer.is_none() => Err(
            AppError::Validation(DomainError::MissingDueOdometer.to_string()),
        ),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::{
            reminder::entity::Reminder,
            vehicle::{
                entity::Vehicle,
                value_objects::{FuelType, Odometer, PlateNumber},
            },
        },
        ports::repositories::{
            CreateReminderParams, CreateVehicleParams, RepositoryError, RepositoryResult,
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
        async fn list_by_user(&self, _uid: Uuid) -> RepositoryResult<Vec<Vehicle>> {
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

        async fn insert(&self, _p: CreateVehicleParams) -> RepositoryResult<Vehicle> {
            Err(RepositoryError::NotFound)
        }

        async fn update(
            &self,
            _id: Uuid,
            _uid: Uuid,
            _p: UpdateVehicleParams,
        ) -> RepositoryResult<Vehicle> {
            Err(RepositoryError::NotFound)
        }

        async fn delete(&self, _id: Uuid, _uid: Uuid) -> RepositoryResult<()> {
            Err(RepositoryError::NotFound)
        }
    }

    struct FakeReminderRepo {
        existing: Reminder,
    }

    #[async_trait]
    impl ReminderRepository for FakeReminderRepo {
        async fn list_by_vehicle(
            &self,
            _vehicle_id: Uuid,
            _user_id: Uuid,
        ) -> RepositoryResult<Vec<Reminder>> {
            Ok(vec![self.existing.clone()])
        }

        async fn find_by_id(
            &self,
            id: Uuid,
            _vehicle_id: Uuid,
            _user_id: Uuid,
        ) -> RepositoryResult<Reminder> {
            if id == self.existing.id {
                Ok(self.existing.clone())
            } else {
                Err(RepositoryError::NotFound)
            }
        }

        async fn insert(&self, _p: CreateReminderParams) -> RepositoryResult<Reminder> {
            Err(RepositoryError::NotFound)
        }

        async fn update(
            &self,
            id: Uuid,
            vehicle_id: Uuid,
            _user_id: Uuid,
            p: UpdateReminderParams,
        ) -> RepositoryResult<Reminder> {
            Ok(Reminder {
                id,
                vehicle_id,
                title: self.existing.title.clone(),
                reminder_type: self.existing.reminder_type.clone(),
                due_date: p.due_date,
                due_odometer: p.due_odometer,
                is_completed: p.is_completed.unwrap_or(self.existing.is_completed),
                notes: p.notes,
                created_at: self.existing.created_at,
            })
        }
    }

    fn make_date_reminder(vehicle_id: Uuid) -> Reminder {
        Reminder {
            id: Uuid::new_v4(),
            vehicle_id,
            title: "Oil change".into(),
            reminder_type: ReminderType::Date,
            due_date: Some("2026-12-01".parse().unwrap()),
            due_odometer: None,
            is_completed: false,
            notes: None,
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn mark_completed_succeeds() {
        let vehicle_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let existing = make_date_reminder(vehicle_id);
        let reminder_id = existing.id;

        let uc = UpdateReminderUseCase {
            repo: Arc::new(FakeReminderRepo { existing }),
            vehicle_repo: Arc::new(FakeVehicleRepo {
                owner_id: user_id,
                vehicle_id,
            }),
        };

        let result = uc
            .execute(PatchReminderInput {
                id: reminder_id,
                vehicle_id,
                user_id,
                is_completed: Some(true),
                due_date: None,
                due_odometer: None,
                notes: None,
            })
            .await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_completed);
    }

    #[tokio::test]
    async fn patch_with_missing_due_date_after_merge_returns_validation_error() {
        let vehicle_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        // Existing date-type reminder has due_date set; if we clear it by providing None
        // but the existing value still carries it — this test verifies patching notes
        // does NOT break existing due_date (merge preserves it)
        let mut existing = make_date_reminder(vehicle_id);
        existing.due_date = None; // simulate corrupted state: date-type with no due_date
        let reminder_id = existing.id;

        let uc = UpdateReminderUseCase {
            repo: Arc::new(FakeReminderRepo { existing }),
            vehicle_repo: Arc::new(FakeVehicleRepo {
                owner_id: user_id,
                vehicle_id,
            }),
        };

        let result = uc
            .execute(PatchReminderInput {
                id: reminder_id,
                vehicle_id,
                user_id,
                is_completed: None,
                due_date: None, // still None after merge → should fail validation
                due_odometer: None,
                notes: Some("Updated notes".into()),
            })
            .await;

        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[tokio::test]
    async fn wrong_owner_returns_not_found() {
        let vehicle_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let intruder_id = Uuid::new_v4();
        let existing = make_date_reminder(vehicle_id);
        let reminder_id = existing.id;

        let uc = UpdateReminderUseCase {
            repo: Arc::new(FakeReminderRepo { existing }),
            vehicle_repo: Arc::new(FakeVehicleRepo {
                owner_id,
                vehicle_id,
            }),
        };

        let result = uc
            .execute(PatchReminderInput {
                id: reminder_id,
                vehicle_id,
                user_id: intruder_id,
                is_completed: Some(true),
                due_date: None,
                due_odometer: None,
                notes: None,
            })
            .await;

        assert!(matches!(result, Err(AppError::NotFound)));
    }
}
