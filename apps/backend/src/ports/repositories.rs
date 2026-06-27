use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::domain::{
    document::entity::Document, expense::entity::Expense, fuel_log::entity::FuelLog,
    reminder::entity::Reminder, service_record::entity::ServiceRecord, user::entity::User,
    vehicle::entity::Vehicle,
};

pub type RepositoryResult<T> = Result<T, RepositoryError>;

/// Errors that can be returned from any repository operation.
///
/// `Database` carries a message string rather than `sqlx::Error` directly so
/// that the `ports` layer stays free of sqlx (layer-boundary rule: ports may
/// only import domain).  The concrete postgres adapters convert
/// `sqlx::Error → RepositoryError::Database(e.to_string())` at the adapter
/// boundary.
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("not found")]
    NotFound,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("database error: {0}")]
    Database(String),
}

// ── User ─────────────────────────────────────────────────────────────────────

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_email(&self, email: &str) -> RepositoryResult<User>;
    async fn find_by_id(&self, id: Uuid) -> RepositoryResult<User>;
    async fn insert(&self, email: &str, password_hash: &str, name: &str) -> RepositoryResult<User>;
}

// ── Vehicle ──────────────────────────────────────────────────────────────────

pub struct CreateVehicleParams {
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

pub struct UpdateVehicleParams {
    pub brand: String,
    pub model: String,
    pub year: i16,
    pub color: Option<String>,
    pub fuel_type: String,
    pub current_odometer: u32,
    pub notes: Option<String>,
}

#[async_trait]
pub trait VehicleRepository: Send + Sync {
    async fn list_by_user(&self, user_id: Uuid) -> RepositoryResult<Vec<Vehicle>>;
    async fn find_by_id(&self, id: Uuid, user_id: Uuid) -> RepositoryResult<Vehicle>;
    async fn insert(&self, params: CreateVehicleParams) -> RepositoryResult<Vehicle>;
    async fn update(
        &self,
        id: Uuid,
        user_id: Uuid,
        params: UpdateVehicleParams,
    ) -> RepositoryResult<Vehicle>;
    async fn delete(&self, id: Uuid, user_id: Uuid) -> RepositoryResult<()>;
}

// ── ServiceRecord ─────────────────────────────────────────────────────────────

pub struct CreateServiceRecordParams {
    pub vehicle_id: Uuid,
    pub service_date: NaiveDate,
    pub odometer: u32,
    pub description: String,
    pub workshop: Option<String>,
    pub cost: Option<Decimal>,
    pub notes: Option<String>,
}

#[async_trait]
pub trait ServiceRecordRepository: Send + Sync {
    async fn list_by_vehicle(
        &self,
        vehicle_id: Uuid,
        user_id: Uuid,
    ) -> RepositoryResult<Vec<ServiceRecord>>;
    async fn insert(&self, params: CreateServiceRecordParams) -> RepositoryResult<ServiceRecord>;
}

// ── FuelLog ───────────────────────────────────────────────────────────────────

pub struct CreateFuelLogParams {
    pub vehicle_id: Uuid,
    pub log_date: NaiveDate,
    pub odometer: u32,
    pub liters: Decimal,
    pub price_per_liter: Decimal,
    pub station: Option<String>,
    pub is_full_tank: bool,
}

#[async_trait]
pub trait FuelLogRepository: Send + Sync {
    async fn list_by_vehicle(
        &self,
        vehicle_id: Uuid,
        user_id: Uuid,
    ) -> RepositoryResult<Vec<FuelLog>>;
    async fn insert(&self, params: CreateFuelLogParams) -> RepositoryResult<FuelLog>;
}

// ── Expense ───────────────────────────────────────────────────────────────────

pub struct CreateExpenseParams {
    pub vehicle_id: Uuid,
    pub expense_date: NaiveDate,
    pub category: String,
    pub description: String,
    pub amount: Decimal,
}

#[async_trait]
pub trait ExpenseRepository: Send + Sync {
    async fn list_by_vehicle(
        &self,
        vehicle_id: Uuid,
        user_id: Uuid,
    ) -> RepositoryResult<Vec<Expense>>;
    async fn insert(&self, params: CreateExpenseParams) -> RepositoryResult<Expense>;
}

// ── Reminder ──────────────────────────────────────────────────────────────────

pub struct CreateReminderParams {
    pub vehicle_id: Uuid,
    pub title: String,
    pub reminder_type: String,
    pub due_date: Option<NaiveDate>,
    pub due_odometer: Option<u32>,
    pub notes: Option<String>,
}

pub struct UpdateReminderParams {
    pub is_completed: Option<bool>,
    pub due_date: Option<NaiveDate>,
    pub due_odometer: Option<u32>,
    pub notes: Option<String>,
}

#[async_trait]
pub trait ReminderRepository: Send + Sync {
    async fn list_by_vehicle(
        &self,
        vehicle_id: Uuid,
        user_id: Uuid,
    ) -> RepositoryResult<Vec<Reminder>>;
    async fn insert(&self, params: CreateReminderParams) -> RepositoryResult<Reminder>;
    async fn update(
        &self,
        id: Uuid,
        vehicle_id: Uuid,
        user_id: Uuid,
        params: UpdateReminderParams,
    ) -> RepositoryResult<Reminder>;
}

// ── Document ──────────────────────────────────────────────────────────────────

pub struct CreateDocumentParams {
    pub vehicle_id: Uuid,
    pub doc_type: String,
    pub title: String,
    pub expiry_date: Option<NaiveDate>,
    pub file_url: Option<String>,
    pub notes: Option<String>,
}

#[async_trait]
pub trait DocumentRepository: Send + Sync {
    async fn list_by_vehicle(
        &self,
        vehicle_id: Uuid,
        user_id: Uuid,
    ) -> RepositoryResult<Vec<Document>>;
    async fn insert(&self, params: CreateDocumentParams) -> RepositoryResult<Document>;
}
