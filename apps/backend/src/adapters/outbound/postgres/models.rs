use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

/// sqlx row struct for users table — separate from domain User entity.
#[derive(Debug, sqlx::FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct VehicleRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub brand: String,
    pub model: String,
    pub year: i16,
    pub plate_number: String,
    pub color: Option<String>,
    pub fuel_type: String,
    pub current_odometer: i32,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ServiceRecordRow {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub service_date: NaiveDate,
    pub odometer: i32,
    pub description: String,
    pub workshop: Option<String>,
    pub cost: Option<Decimal>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct FuelLogRow {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub log_date: NaiveDate,
    pub odometer: i32,
    pub liters: Decimal,
    pub price_per_liter: Decimal,
    pub total_cost: Decimal,
    pub station: Option<String>,
    pub is_full_tank: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ExpenseRow {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub expense_date: NaiveDate,
    pub category: String,
    pub description: String,
    pub amount: Decimal,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ReminderRow {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub title: String,
    pub reminder_type: String,
    pub due_date: Option<NaiveDate>,
    pub due_odometer: Option<i32>,
    pub is_completed: bool,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct DocumentRow {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub doc_type: String,
    pub title: String,
    pub expiry_date: Option<NaiveDate>,
    pub file_url: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}
