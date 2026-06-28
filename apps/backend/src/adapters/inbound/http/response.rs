//! The standardized success response envelope.
//!
//! Every successful response is wrapped as `{ "meta": { "request_id": ... },
//! "data": <payload> }` with the appropriate HTTP status. `data` and `error` are
//! mutually exclusive across the contract — success bodies carry `data`, error
//! bodies (see [`super::errors`]) carry `error`. The HTTP status line is
//! authoritative; it is intentionally NOT duplicated in the body.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use super::request_id::current_request_id;

/// Response metadata. Currently the request correlation id; collection endpoints
/// will gain a `pagination` field here when pagination is implemented.
#[derive(Debug, Serialize)]
pub struct Meta {
    pub request_id: String,
}

impl Meta {
    /// Build metadata for the current request from the request-id task-local.
    pub fn current() -> Self {
        Self {
            request_id: current_request_id(),
        }
    }
}

#[derive(Serialize)]
struct Envelope<T: Serialize> {
    meta: Meta,
    data: T,
}

/// A success response carrying `data` inside the standard envelope. Construct with
/// [`ApiResponse::ok`] (200) or [`ApiResponse::created`] (201); both stamp the
/// current request id into `meta`.
pub struct ApiResponse<T: Serialize> {
    status: StatusCode,
    body: Envelope<T>,
}

impl<T: Serialize> ApiResponse<T> {
    /// 200 OK with the payload under `data`.
    pub fn ok(data: T) -> Self {
        Self::with_status(StatusCode::OK, data)
    }

    /// 201 Created with the payload under `data`.
    pub fn created(data: T) -> Self {
        Self::with_status(StatusCode::CREATED, data)
    }

    fn with_status(status: StatusCode, data: T) -> Self {
        Self {
            status,
            body: Envelope {
                meta: Meta::current(),
                data,
            },
        }
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        (self.status, Json(self.body)).into_response()
    }
}
