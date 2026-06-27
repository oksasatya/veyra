use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::application::errors::AppError;

/// Converts [`AppError`] to an axum HTTP response. This is the single choke
/// point where domain/application errors are mapped to status codes — handlers
/// never construct error responses themselves.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::Conflict(_) => (StatusCode::CONFLICT, self.to_string()),
            AppError::Validation(_) => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()),
            AppError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal server error".into(),
            ),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}
