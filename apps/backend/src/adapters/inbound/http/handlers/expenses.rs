use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    adapters::inbound::http::{
        dto::expense::{CreateExpenseRequest, ExpenseResponse},
        response::ApiResponse,
    },
    application::{
        errors::AppError,
        expense::{
            create::{CreateExpenseInput, CreateExpenseUseCase},
            list::ListExpensesUseCase,
        },
    },
    bootstrap::state::AppState,
    domain::expense::entity::Expense,
};

/// Maps a domain `Expense` to the HTTP response DTO.
fn to_response(expense: Expense) -> ExpenseResponse {
    ExpenseResponse {
        id: expense.id.to_string(),
        vehicle_id: expense.vehicle_id.to_string(),
        expense_date: expense.expense_date,
        category: expense.category.as_str().to_string(),
        amount: expense.amount.to_string(),
        description: expense.description,
    }
}

/// GET /vehicles/{vehicle_id}/expenses — list all expenses for a vehicle.
///
/// Returns 404 when the vehicle is not found or not owned by the caller.
pub async fn list(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(vehicle_id): Path<Uuid>,
) -> Result<ApiResponse<Vec<ExpenseResponse>>, AppError> {
    let uc = ListExpensesUseCase {
        repo: state.expense_repo.clone(),
        vehicle_repo: state.vehicle_repo.clone(),
    };
    let expenses = uc.execute(vehicle_id, user_id).await?;
    Ok(ApiResponse::ok(
        expenses.into_iter().map(to_response).collect(),
    ))
}

/// POST /vehicles/{vehicle_id}/expenses — create an expense for a vehicle.
///
/// Returns 201 Created with the created expense, 404 if the vehicle is not
/// owned by the caller, or 422 if the category is invalid.
pub async fn create(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(vehicle_id): Path<Uuid>,
    Json(body): Json<CreateExpenseRequest>,
) -> Result<ApiResponse<ExpenseResponse>, AppError> {
    let uc = CreateExpenseUseCase {
        repo: state.expense_repo.clone(),
        vehicle_repo: state.vehicle_repo.clone(),
    };
    let expense = uc
        .execute(CreateExpenseInput {
            vehicle_id,
            user_id,
            expense_date: body.expense_date,
            category: body.category,
            description: body.description,
            amount: body.amount,
        })
        .await?;
    Ok(ApiResponse::created(to_response(expense)))
}
