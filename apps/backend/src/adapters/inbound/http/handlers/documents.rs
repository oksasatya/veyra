use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    adapters::inbound::http::{
        dto::document::{CreateDocumentRequest, DocumentResponse},
        response::ApiResponse,
    },
    application::{
        document::{
            create::{CreateDocumentInput, CreateDocumentUseCase},
            list::ListDocumentsUseCase,
        },
        errors::AppError,
    },
    bootstrap::state::AppState,
    domain::document::entity::Document,
};

/// Maps a domain `Document` to the HTTP response DTO.
fn to_response(doc: Document) -> DocumentResponse {
    DocumentResponse {
        id: doc.id.to_string(),
        vehicle_id: doc.vehicle_id.to_string(),
        doc_type: doc.doc_type.as_str().to_string(),
        title: doc.title,
        expiry_date: doc.expiry_date,
        file_url: doc.file_url,
    }
}

/// GET /vehicles/{vehicle_id}/documents — list all documents for a vehicle.
///
/// Returns 404 when the vehicle is not found or not owned by the caller.
pub async fn list(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(vehicle_id): Path<Uuid>,
) -> Result<ApiResponse<Vec<DocumentResponse>>, AppError> {
    let uc = ListDocumentsUseCase {
        repo: state.document_repo.clone(),
        vehicle_repo: state.vehicle_repo.clone(),
    };
    let documents = uc.execute(vehicle_id, user_id).await?;
    Ok(ApiResponse::ok(
        documents.into_iter().map(to_response).collect(),
    ))
}

/// POST /vehicles/{vehicle_id}/documents — create a document for a vehicle.
///
/// Returns 201 Created with the created document, 404 if the vehicle is not
/// owned by the caller, or 422 if the doc_type is invalid.
pub async fn create(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(vehicle_id): Path<Uuid>,
    Json(body): Json<CreateDocumentRequest>,
) -> Result<ApiResponse<DocumentResponse>, AppError> {
    let uc = CreateDocumentUseCase {
        repo: state.document_repo.clone(),
        vehicle_repo: state.vehicle_repo.clone(),
    };
    let doc = uc
        .execute(CreateDocumentInput {
            vehicle_id,
            user_id,
            doc_type: body.doc_type,
            title: body.title,
            expiry_date: body.expiry_date,
            file_url: body.file_url,
            notes: body.notes,
        })
        .await?;
    Ok(ApiResponse::created(to_response(doc)))
}
