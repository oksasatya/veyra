use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    adapters::inbound::http::{
        dto::service_record::{CreateServiceRecordRequest, ServiceRecordResponse},
        response::ApiResponse,
    },
    application::{
        errors::AppError,
        service_record::{
            create::{CreateServiceRecordInput, CreateServiceRecordUseCase},
            list::ListServiceRecordsUseCase,
        },
    },
    bootstrap::state::AppState,
    domain::service_record::entity::ServiceRecord,
};

/// Maps a domain `ServiceRecord` to the HTTP response DTO.
fn to_response(r: ServiceRecord) -> ServiceRecordResponse {
    ServiceRecordResponse {
        id: r.id.to_string(),
        vehicle_id: r.vehicle_id.to_string(),
        service_date: r.service_date,
        odometer: r.odometer,
        description: r.description,
        workshop: r.workshop,
        cost: r.cost.map(|c| c.to_string()),
        notes: r.notes,
    }
}

/// GET /vehicles/{vehicle_id}/services — list all service records for a vehicle.
///
/// Returns 404 when the vehicle is not found or not owned by the caller.
pub async fn list(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(vehicle_id): Path<Uuid>,
) -> Result<ApiResponse<Vec<ServiceRecordResponse>>, AppError> {
    let uc = ListServiceRecordsUseCase {
        repo: state.service_record_repo.clone(),
        vehicle_repo: state.vehicle_repo.clone(),
    };
    let records = uc.execute(vehicle_id, user_id).await?;
    Ok(ApiResponse::ok(
        records.into_iter().map(to_response).collect(),
    ))
}

/// POST /vehicles/{vehicle_id}/services — create a service record for a vehicle.
///
/// Returns 201 Created with the created record, or 404 if the vehicle is not
/// owned by the caller.
pub async fn create(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(vehicle_id): Path<Uuid>,
    Json(body): Json<CreateServiceRecordRequest>,
) -> Result<ApiResponse<ServiceRecordResponse>, AppError> {
    let uc = CreateServiceRecordUseCase {
        repo: state.service_record_repo.clone(),
        vehicle_repo: state.vehicle_repo.clone(),
    };
    let record = uc
        .execute(CreateServiceRecordInput {
            vehicle_id,
            user_id,
            service_date: body.service_date,
            odometer: body.odometer,
            description: body.description,
            workshop: body.workshop,
            cost: body.cost,
            notes: body.notes,
        })
        .await?;
    Ok(ApiResponse::created(to_response(record)))
}
