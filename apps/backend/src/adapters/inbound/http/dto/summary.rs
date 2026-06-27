use serde::Serialize;

/// HTTP response DTO for the vehicle dashboard summary endpoint.
///
/// Decimal values are serialised as strings to avoid floating-point precision
/// loss across JSON consumers — consistent with every other monetary field in
/// the API (service_records, fuel_logs, expenses).
#[derive(Debug, Serialize)]
pub struct VehicleSummaryResponse {
    pub vehicle_id: String,
    pub current_odometer: i32,
    pub total_services: i64,
    pub total_service_cost: String,
    pub total_refuels: i64,
    pub total_fuel_cost: String,
    pub total_expenses: String,
    pub upcoming_reminders: i64,
}
