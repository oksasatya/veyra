use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ServiceRecord {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub service_date: NaiveDate,
    pub odometer: u32,
    pub description: String,
    pub workshop: Option<String>,
    pub cost: Option<Decimal>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}
