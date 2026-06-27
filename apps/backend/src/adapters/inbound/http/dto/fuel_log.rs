use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateFuelLogRequest {
    pub log_date: NaiveDate,
    pub odometer: u32,
    pub liters: Decimal,
    pub price_per_liter: Decimal,
    pub station: Option<String>,
    pub is_full_tank: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct FuelLogResponse {
    pub id: String,
    pub vehicle_id: String,
    pub log_date: NaiveDate,
    pub odometer: u32,
    pub liters: String,
    pub price_per_liter: String,
    /// Serialised as a string to avoid floating-point precision loss.
    pub total_cost: String,
    pub station: Option<String>,
    pub is_full_tank: bool,
}

#[derive(Debug, Serialize)]
pub struct FuelLogListResponse {
    pub logs: Vec<FuelLogResponse>,
}
