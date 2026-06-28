use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    adapters::inbound::http::{
        dto::fuel_log::{CreateFuelLogRequest, FuelLogResponse},
        response::ApiResponse,
    },
    application::{
        errors::AppError,
        fuel_log::{
            create::{CreateFuelLogInput, CreateFuelLogUseCase},
            list::ListFuelLogsUseCase,
        },
    },
    bootstrap::state::AppState,
    domain::fuel_log::entity::FuelLog,
};

/// Maps a domain `FuelLog` to the HTTP response DTO.
fn to_response(log: FuelLog) -> FuelLogResponse {
    FuelLogResponse {
        id: log.id.to_string(),
        vehicle_id: log.vehicle_id.to_string(),
        log_date: log.log_date,
        odometer: log.odometer,
        liters: log.liters.to_string(),
        price_per_liter: log.price_per_liter.to_string(),
        total_cost: log.total_cost.to_string(),
        station: log.station,
        is_full_tank: log.is_full_tank,
    }
}

/// GET /vehicles/{vehicle_id}/fuel-logs — list all fuel logs for a vehicle.
///
/// Returns 404 when the vehicle is not found or not owned by the caller.
pub async fn list(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(vehicle_id): Path<Uuid>,
) -> Result<ApiResponse<Vec<FuelLogResponse>>, AppError> {
    let uc = ListFuelLogsUseCase {
        repo: state.fuel_log_repo.clone(),
        vehicle_repo: state.vehicle_repo.clone(),
    };
    let logs = uc.execute(vehicle_id, user_id).await?;
    Ok(ApiResponse::ok(logs.into_iter().map(to_response).collect()))
}

/// POST /vehicles/{vehicle_id}/fuel-logs — create a fuel log for a vehicle.
///
/// Returns 201 Created with the created log, or 404 if the vehicle is not
/// owned by the caller.
pub async fn create(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(vehicle_id): Path<Uuid>,
    Json(body): Json<CreateFuelLogRequest>,
) -> Result<ApiResponse<FuelLogResponse>, AppError> {
    let uc = CreateFuelLogUseCase {
        repo: state.fuel_log_repo.clone(),
        vehicle_repo: state.vehicle_repo.clone(),
    };
    let log = uc
        .execute(CreateFuelLogInput {
            vehicle_id,
            user_id,
            log_date: body.log_date,
            odometer: body.odometer,
            liters: body.liters,
            price_per_liter: body.price_per_liter,
            station: body.station,
            is_full_tank: body.is_full_tank.unwrap_or(true),
        })
        .await?;
    Ok(ApiResponse::created(to_response(log)))
}
