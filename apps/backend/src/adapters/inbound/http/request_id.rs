//! Per-request correlation id.
//!
//! A middleware generates a fresh id for every request, exposes it to response
//! builders via a task-local, and echoes it back as the `X-Request-Id` header.
//! Because the id lives in a task-local set for the whole request scope, BOTH the
//! success envelope ([`super::response::ApiResponse`]) and the error envelope
//! ([`super::errors`]) can read it without threading an extractor through every
//! handler — the single source of truth for `meta.request_id`.

use axum::{
    extract::Request,
    http::{HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

/// The header name we echo the request id back on.
pub const X_REQUEST_ID: &str = "x-request-id";

tokio::task_local! {
    static REQUEST_ID: String;
}

/// The current request's id, or `"unknown"` when called outside a request scope
/// (e.g. a background task). Cheap clone of the task-local string.
pub fn current_request_id() -> String {
    REQUEST_ID
        .try_with(String::clone)
        .unwrap_or_else(|_| "unknown".to_string())
}

/// Middleware: generate a request id, run the request inside its task-local scope,
/// and set the `X-Request-Id` response header. Wire this as the OUTERMOST layer so
/// the id is available to every handler and to error responses alike.
pub async fn propagate_request_id(request: Request, next: Next) -> Response {
    let id = Uuid::new_v4().to_string();
    let header_value = HeaderValue::from_str(&id).ok();

    let mut response = REQUEST_ID.scope(id, next.run(request)).await;

    if let Some(value) = header_value {
        response
            .headers_mut()
            .insert(HeaderName::from_static(X_REQUEST_ID), value);
    }
    response
}
