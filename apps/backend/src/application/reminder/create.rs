use std::sync::Arc;

use chrono::NaiveDate;
use uuid::Uuid;

use crate::{
    application::errors::AppError,
    domain::{
        errors::DomainError,
        reminder::entity::{Reminder, ReminderType},
    },
    ports::repositories::{CreateReminderParams, ReminderRepository, VehicleRepository},
};

pub struct CreateReminderUseCase {
    pub repo: Arc<dyn ReminderRepository>,
    pub vehicle_repo: Arc<dyn VehicleRepository>,
}

pub struct CreateReminderInput {
    pub vehicle_id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub reminder_type: String,
    pub due_date: Option<NaiveDate>,
    pub due_odometer: Option<u32>,
    pub notes: Option<String>,
}

impl CreateReminderUseCase {
    /// Verifies vehicle ownership, validates `reminder_type`, then validates
    /// that at least one due trigger is set (date or odometer per type).
    /// Returns `AppError::NotFound` if vehicle is not owned by caller,
    /// or `AppError::Validation` for invalid type or missing trigger fields.
    pub async fn execute(&self, input: CreateReminderInput) -> Result<Reminder, AppError> {
        // Parse and validate reminder_type before hitting the database
        let reminder_type = ReminderType::parse(&input.reminder_type)
            .ok_or_else(|| AppError::Validation(format!("unknown reminder_type: {}", input.reminder_type)))?;

        // Validate due trigger fields based on type
        validate_due_triggers(&reminder_type, input.due_date, input.due_odometer)?;

        // Ownership guard: vehicle must belong to the caller
        self.vehicle_repo
            .find_by_id(input.vehicle_id, input.user_id)
            .await
            .map_err(AppError::from)?;

        self.repo
            .insert(CreateReminderParams {
                vehicle_id: input.vehicle_id,
                title: input.title,
                reminder_type: input.reminder_type,
                due_date: input.due_date,
                due_odometer: input.due_odometer,
                notes: input.notes,
            })
            .await
            .map_err(AppError::from)
    }
}

/// Validates that the required due trigger fields are present for the given type.
///
/// - `Date`  → `due_date` must be `Some`
/// - `Odometer` → `due_odometer` must be `Some`
/// - `Both`  → both must be `Some`
fn validate_due_triggers(
    reminder_type: &ReminderType,
    due_date: Option<NaiveDate>,
    due_odometer: Option<u32>,
) -> Result<(), AppError> {
    match reminder_type {
        ReminderType::Date | ReminderType::Both if due_date.is_none() => {
            Err(AppError::Validation(DomainError::MissingDueDate.to_string()))
        }
        ReminderType::Odometer | ReminderType::Both if due_odometer.is_none() => {
            Err(AppError::Validation(DomainError::MissingDueOdometer.to_string()))
        }
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
            UpdateReminderParams, UpdateVehicleParams,
        },
    };
    use async_trait::async_trait;
    use chrono::Utc;

    // ── Fakes ────────────────────────────────────────────────────────────────

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

        async fn insert(&self, _p: CreateVehicleParams) -> RepositoryResult<Vehicle> {
            Err(RepositoryError::NotFound)
        }

        async fn update(
            &self,
            _id: Uuid,
            _user_id: Uuid,
            _p: UpdateVehicleParams,
        ) -> RepositoryResult<Vehicle> {
            Err(RepositoryError::NotFound)
        }

        async fn delete(&self, _id: Uuid, _user_id: Uuid) -> RepositoryResult<()> {
            Err(RepositoryError::NotFound)
        }
    }

    struct FakeReminderRepo;

    #[async_trait]
    impl ReminderRepository for FakeReminderRepo {
        async fn list_by_vehicle(
            &self,
            _vehicle_id: Uuid,
            _user_id: Uuid,
        ) -> RepositoryResult<Vec<Reminder>> {
            Ok(vec![])
        }

        async fn insert(&self, p: CreateReminderParams) -> RepositoryResult<Reminder> {
            Ok(Reminder {
                id: Uuid::new_v4(),
                vehicle_id: p.vehicle_id,
                title: p.title,
                reminder_type: ReminderType::parse(&p.reminder_type).unwrap(),
                due_date: p.due_date,
                due_odometer: p.due_odometer,
                is_completed: false,
                notes: p.notes,
                created_at: Utc::now(),
            })
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
                title: "Test".into(),
                reminder_type: ReminderType::Date,
                due_date: p.due_date,
                due_odometer: p.due_odometer,
                is_completed: p.is_completed.unwrap_or(false),
                notes: p.notes,
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

    fn make_uc(owner_id: Uuid, vehicle_id: Uuid) -> CreateReminderUseCase {
        CreateReminderUseCase {
            repo: Arc::new(FakeReminderRepo),
            vehicle_repo: Arc::new(FakeVehicleRepo { owner_id, vehicle_id }),
        }
    }

    // ── Tests ────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn date_type_without_due_date_returns_validation_error() {
        let vid = Uuid::new_v4();
        let uid = Uuid::new_v4();
        let uc = make_uc(uid, vid);

        let result = uc.execute(CreateReminderInput {
            vehicle_id: vid,
            user_id: uid,
            title: "Oil change".into(),
            reminder_type: "date".into(),
            due_date: None,        // missing — should fail
            due_odometer: None,
            notes: None,
        }).await;

        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[tokio::test]
    async fn odometer_type_without_due_odometer_returns_validation_error() {
        let vid = Uuid::new_v4();
        let uid = Uuid::new_v4();
        let uc = make_uc(uid, vid);

        let result = uc.execute(CreateReminderInput {
            vehicle_id: vid,
            user_id: uid,
            title: "Oil change".into(),
            reminder_type: "odometer".into(),
            due_date: None,
            due_odometer: None,    // missing — should fail
            notes: None,
        }).await;

        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[tokio::test]
    async fn both_type_without_due_date_returns_validation_error() {
        let vid = Uuid::new_v4();
        let uid = Uuid::new_v4();
        let uc = make_uc(uid, vid);

        let result = uc.execute(CreateReminderInput {
            vehicle_id: vid,
            user_id: uid,
            title: "Full service".into(),
            reminder_type: "both".into(),
            due_date: None,         // missing — should fail
            due_odometer: Some(50_000),
            notes: None,
        }).await;

        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[tokio::test]
    async fn both_type_without_due_odometer_returns_validation_error() {
        let vid = Uuid::new_v4();
        let uid = Uuid::new_v4();
        let uc = make_uc(uid, vid);

        let result = uc.execute(CreateReminderInput {
            vehicle_id: vid,
            user_id: uid,
            title: "Full service".into(),
            reminder_type: "both".into(),
            due_date: Some("2026-12-01".parse().unwrap()),
            due_odometer: None,     // missing — should fail
            notes: None,
        }).await;

        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[tokio::test]
    async fn date_type_with_due_date_succeeds() {
        let vid = Uuid::new_v4();
        let uid = Uuid::new_v4();
        let uc = make_uc(uid, vid);

        let result = uc.execute(CreateReminderInput {
            vehicle_id: vid,
            user_id: uid,
            title: "Oil change".into(),
            reminder_type: "date".into(),
            due_date: Some("2026-12-01".parse().unwrap()),
            due_odometer: None,
            notes: None,
        }).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().reminder_type, ReminderType::Date);
    }

    #[tokio::test]
    async fn both_type_with_both_fields_succeeds() {
        let vid = Uuid::new_v4();
        let uid = Uuid::new_v4();
        let uc = make_uc(uid, vid);

        let result = uc.execute(CreateReminderInput {
            vehicle_id: vid,
            user_id: uid,
            title: "Full service".into(),
            reminder_type: "both".into(),
            due_date: Some("2026-12-01".parse().unwrap()),
            due_odometer: Some(50_000),
            notes: None,
        }).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn unknown_type_returns_validation_error() {
        let vid = Uuid::new_v4();
        let uid = Uuid::new_v4();
        let uc = make_uc(uid, vid);

        let result = uc.execute(CreateReminderInput {
            vehicle_id: vid,
            user_id: uid,
            title: "Test".into(),
            reminder_type: "invalid_type".into(),
            due_date: None,
            due_odometer: None,
            notes: None,
        }).await;

        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[tokio::test]
    async fn wrong_owner_returns_not_found() {
        let vid = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let intruder_id = Uuid::new_v4();
        let uc = make_uc(owner_id, vid);

        let result = uc.execute(CreateReminderInput {
            vehicle_id: vid,
            user_id: intruder_id,
            title: "Test".into(),
            reminder_type: "date".into(),
            due_date: Some("2026-12-01".parse().unwrap()),
            due_odometer: None,
            notes: None,
        }).await;

        assert!(matches!(result, Err(AppError::NotFound)));
    }
}
