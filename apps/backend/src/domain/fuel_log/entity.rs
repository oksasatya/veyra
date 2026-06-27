use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct FuelLog {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub log_date: NaiveDate,
    pub odometer: u32,
    pub liters: Decimal,
    pub price_per_liter: Decimal,
    pub total_cost: Decimal,
    pub station: Option<String>,
    pub is_full_tank: bool,
    pub created_at: DateTime<Utc>,
}
