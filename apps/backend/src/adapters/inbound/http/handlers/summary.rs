use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    adapters::inbound::http::dto::summary::VehicleSummaryResponse,
    application::{errors::AppError, summary::get::GetSummaryUseCase},
    bootstrap::state::AppState,
};

/// GET /vehicles/{vehicle_id}/summary — return aggregated statistics for one vehicle.
///
/// Requires a valid Bearer JWT. Returns 404 if the vehicle does not exist or is
/// not owned by the requesting user.
pub async fn get_summary(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(vehicle_id): Path<Uuid>,
) -> Result<Json<VehicleSummaryResponse>, AppError> {
    let uc = GetSummaryUseCase { pool: state.pool.clone() };
    let s = uc.execute(vehicle_id, user_id).await?;
    Ok(Json(VehicleSummaryResponse {
        vehicle_id: s.vehicle_id.to_string(),
        current_odometer: s.current_odometer,
        total_services: s.total_services,
        total_service_cost: s.total_service_cost.to_string(),
        total_refuels: s.total_refuels,
        total_fuel_cost: s.total_fuel_cost.to_string(),
        total_expenses: s.total_expenses.to_string(),
        upcoming_reminders: s.upcoming_reminders,
    }))
}
