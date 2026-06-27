use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateVehicleRequest {
    pub brand: String,
    pub model: String,
    pub year: i16,
    pub plate_number: String,
    pub color: Option<String>,
    pub fuel_type: String,
    pub current_odometer: u32,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateVehicleRequest {
    pub brand: String,
    pub model: String,
    pub year: i16,
    pub color: Option<String>,
    pub fuel_type: String,
    pub current_odometer: u32,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct VehicleResponse {
    pub id: String,
    pub brand: String,
    pub model: String,
    pub year: i16,
    pub plate_number: String,
    pub color: Option<String>,
    pub fuel_type: String,
    pub current_odometer: u32,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct VehicleListResponse {
    pub vehicles: Vec<VehicleResponse>,
}
