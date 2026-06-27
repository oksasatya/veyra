use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    adapters::inbound::http::dto::reminder::{
        CreateReminderRequest, PatchReminderRequest, ReminderListResponse, ReminderResponse,
    },
    application::{
        errors::AppError,
        reminder::{
            create::{CreateReminderInput, CreateReminderUseCase},
            list::ListRemindersUseCase,
            update::{PatchReminderInput, UpdateReminderUseCase},
        },
    },
    bootstrap::state::AppState,
    domain::reminder::entity::Reminder,
};

/// Maps a domain `Reminder` to the HTTP response DTO.
fn to_response(r: Reminder) -> ReminderResponse {
    ReminderResponse {
        id: r.id.to_string(),
        vehicle_id: r.vehicle_id.to_string(),
        title: r.title,
        reminder_type: r.reminder_type.as_str().to_string(),
        due_date: r.due_date,
        due_odometer: r.due_odometer,
        is_completed: r.is_completed,
        notes: r.notes,
    }
}

/// GET /vehicles/{vehicle_id}/reminders — list all reminders for a vehicle.
pub async fn list(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(vehicle_id): Path<Uuid>,
) -> Result<Json<ReminderListResponse>, AppError> {
    let uc = ListRemindersUseCase {
        repo: state.reminder_repo.clone(),
        vehicle_repo: state.vehicle_repo.clone(),
    };
    let reminders = uc.execute(vehicle_id, user_id).await?;
    Ok(Json(ReminderListResponse {
        reminders: reminders.into_iter().map(to_response).collect(),
    }))
}

/// POST /vehicles/{vehicle_id}/reminders — create a reminder for a vehicle.
///
/// Returns 201 Created, 404 if the vehicle is not owned by the caller,
/// or 422 if the reminder_type is invalid or required due fields are missing.
pub async fn create(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(vehicle_id): Path<Uuid>,
    Json(body): Json<CreateReminderRequest>,
) -> Result<(StatusCode, Json<ReminderResponse>), AppError> {
    let uc = CreateReminderUseCase {
        repo: state.reminder_repo.clone(),
        vehicle_repo: state.vehicle_repo.clone(),
    };
    let reminder = uc
        .execute(CreateReminderInput {
            vehicle_id,
            user_id,
            title: body.title,
            reminder_type: body.reminder_type,
            due_date: body.due_date,
            due_odometer: body.due_odometer,
            notes: body.notes,
        })
        .await?;
    Ok((StatusCode::CREATED, Json(to_response(reminder))))
}

/// PATCH /vehicles/{vehicle_id}/reminders/{id} — partial update.
///
/// Typically used to mark a reminder complete or update due fields.
/// Returns 404 if the vehicle or reminder is not owned by the caller,
/// or 422 if the patch would leave the reminder in an invalid state.
pub async fn patch(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path((vehicle_id, id)): Path<(Uuid, Uuid)>,
    Json(body): Json<PatchReminderRequest>,
) -> Result<Json<ReminderResponse>, AppError> {
    let uc = UpdateReminderUseCase {
        repo: state.reminder_repo.clone(),
        vehicle_repo: state.vehicle_repo.clone(),
    };
    let reminder = uc
        .execute(PatchReminderInput {
            id,
            vehicle_id,
            user_id,
            is_completed: body.is_completed,
            due_date: body.due_date,
            due_odometer: body.due_odometer,
            notes: body.notes,
        })
        .await?;
    Ok(Json(to_response(reminder)))
}
