use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    adapters::inbound::http::dto::vehicle::{
        CreateVehicleRequest, UpdateVehicleRequest, VehicleListResponse, VehicleResponse,
    },
    application::{
        errors::AppError,
        vehicle::{
            create::{CreateVehicleInput, CreateVehicleUseCase},
            delete::DeleteVehicleUseCase,
            get::GetVehicleUseCase,
            list::ListVehiclesUseCase,
            update::{UpdateVehicleInput, UpdateVehicleUseCase},
        },
    },
    bootstrap::state::AppState,
    domain::vehicle::entity::Vehicle,
};

fn vehicle_to_response(v: Vehicle) -> VehicleResponse {
    VehicleResponse {
        id: v.id.to_string(),
        brand: v.brand,
        model: v.model,
        year: v.year,
        plate_number: v.plate_number.as_str().to_string(),
        color: v.color,
        fuel_type: v.fuel_type.as_str().to_string(),
        current_odometer: v.current_odometer.value(),
        notes: v.notes,
    }
}

/// GET /vehicles — list all vehicles owned by the authenticated user.
pub async fn list(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<VehicleListResponse>, AppError> {
    let uc = ListVehiclesUseCase {
        repo: state.vehicle_repo.clone(),
    };
    let vehicles = uc.execute(user_id).await?;
    Ok(Json(VehicleListResponse {
        vehicles: vehicles.into_iter().map(vehicle_to_response).collect(),
    }))
}

/// POST /vehicles — create a new vehicle for the authenticated user.
/// Returns 201 Created with the created vehicle.
pub async fn create(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Json(body): Json<CreateVehicleRequest>,
) -> Result<(StatusCode, Json<VehicleResponse>), AppError> {
    let uc = CreateVehicleUseCase {
        repo: state.vehicle_repo.clone(),
    };
    let vehicle = uc
        .execute(CreateVehicleInput {
            user_id,
            brand: body.brand,
            model: body.model,
            year: body.year,
            plate_number: body.plate_number,
            color: body.color,
            fuel_type: body.fuel_type,
            current_odometer: body.current_odometer,
            notes: body.notes,
        })
        .await?;
    Ok((StatusCode::CREATED, Json(vehicle_to_response(vehicle))))
}

/// GET /vehicles/:id — get a specific vehicle by ID.
/// Returns 404 if the vehicle does not exist or is not owned by the caller.
pub async fn get(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<Uuid>,
) -> Result<Json<VehicleResponse>, AppError> {
    let uc = GetVehicleUseCase {
        repo: state.vehicle_repo.clone(),
    };
    let vehicle = uc.execute(id, user_id).await?;
    Ok(Json(vehicle_to_response(vehicle)))
}

/// PUT /vehicles/:id — update a vehicle owned by the authenticated user.
/// Returns 404 if the vehicle does not exist or is not owned by the caller.
pub async fn update(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateVehicleRequest>,
) -> Result<Json<VehicleResponse>, AppError> {
    let uc = UpdateVehicleUseCase {
        repo: state.vehicle_repo.clone(),
    };
    let vehicle = uc
        .execute(
            id,
            user_id,
            UpdateVehicleInput {
                brand: body.brand,
                model: body.model,
                year: body.year,
                color: body.color,
                fuel_type: body.fuel_type,
                current_odometer: body.current_odometer,
                notes: body.notes,
            },
        )
        .await?;
    Ok(Json(vehicle_to_response(vehicle)))
}

/// DELETE /vehicles/:id — delete a vehicle owned by the authenticated user.
/// Returns 204 No Content on success, 404 if not found/not owned.
pub async fn delete(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let uc = DeleteVehicleUseCase {
        repo: state.vehicle_repo.clone(),
    };
    uc.execute(id, user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
