use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::value_objects::{FuelType, Odometer, PlateNumber};

#[derive(Debug, Clone)]
pub struct Vehicle {
    pub id: Uuid,
    pub user_id: Uuid,
    pub brand: String,
    pub model: String,
    pub year: i16,
    pub plate_number: PlateNumber,
    pub color: Option<String>,
    pub fuel_type: FuelType,
    pub current_odometer: Odometer,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
