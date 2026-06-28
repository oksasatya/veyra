use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use super::{request_id::current_request_id, response::Meta};
use crate::application::errors::AppError;

/// The error body nested under `error` in the envelope. `code` is the stable,
/// machine-readable identifier the client localizes from; `message` is English
/// developer prose for logs/debugging — clients must NOT display it.
#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
}

#[derive(Serialize)]
struct ErrorEnvelope {
    meta: Meta,
    error: ErrorBody,
}

const INTERNAL_MESSAGE: &str = "internal server error";

/// Converts [`AppError`] to a standardized error envelope:
/// `{ "meta": { "request_id": ... }, "error": { "code", "message" } }`.
///
/// This is the single choke point where application errors are mapped to HTTP
/// status codes — handlers never construct error responses themselves. 5xx detail
/// is masked from the client (logged instead); 4xx messages are safe developer prose.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
            AppError::Conflict { .. } => StatusCode::CONFLICT,
            AppError::Validation { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let code = self.code().as_str();

        if status.is_server_error() {
            // Log the real cause; never leak internal detail to the client.
            tracing::error!(error = %self, %code, "server error");
        } else {
            tracing::info!(error = %self, %code, "request error");
        }

        // Mask ONLY the catch-all 500 (it wraps arbitrary causes — SQL, etc.). Other
        // variants carry their own non-leaky, safe message (e.g. 503 "service unavailable").
        let message = match &self {
            AppError::Internal(_) => INTERNAL_MESSAGE.to_string(),
            _ => self.to_string(),
        };

        let body = ErrorEnvelope {
            meta: Meta {
                request_id: current_request_id(),
            },
            error: ErrorBody { code, message },
        };

        (status, Json(body)).into_response()
    }
}
