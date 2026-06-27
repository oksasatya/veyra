use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateServiceRecordRequest {
    pub service_date: NaiveDate,
    pub odometer: u32,
    pub description: String,
    pub workshop: Option<String>,
    pub cost: Option<Decimal>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ServiceRecordResponse {
    pub id: String,
    pub vehicle_id: String,
    pub service_date: NaiveDate,
    pub odometer: u32,
    pub description: String,
    pub workshop: Option<String>,
    /// Serialised as a string to avoid floating-point precision loss.
    pub cost: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ServiceRecordListResponse {
    pub records: Vec<ServiceRecordResponse>,
}
