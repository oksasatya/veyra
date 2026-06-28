use axum::{
    middleware,
    routing::{get, patch, post},
    Router,
};

use crate::bootstrap::state::AppState;

use super::{
    handlers::{
        auth as auth_handlers, documents as document_handlers, expenses as expense_handlers,
        fuel_logs as fuel_log_handlers, health::health, reminders as reminder_handlers,
        service_records as service_record_handlers, summary as summary_handlers,
        vehicles as vehicle_handlers,
    },
    middleware::auth::require_auth,
    request_id::propagate_request_id,
};

/// Builds the axum router with two route groups:
///
/// - **Open** (`/health`, `/auth/register`, `/auth/login`, `/auth/refresh`,
///   `/auth/logout`) — no auth layer. Register/login open a session; refresh and
///   logout authenticate via the opaque refresh token in the JSON body, so they
///   keep working once the access token has expired.
/// - **Protected** (`/me`, `/vehicles/*`) — `require_auth` validates the
///   `Authorization: Bearer` access token + sid-revocation, injecting `Uuid`.
///
/// The API is consumed only by the native mobile client over bearer tokens, so
/// there is no cookie, CSRF, or CORS surface.
pub fn build(state: AppState) -> Router {
    let protected = Router::new()
        .route("/me", get(auth_handlers::me).patch(auth_handlers::patch_me))
        .route(
            "/vehicles",
            get(vehicle_handlers::list).post(vehicle_handlers::create),
        )
        .route(
            "/vehicles/{id}",
            get(vehicle_handlers::get)
                .put(vehicle_handlers::update)
                .delete(vehicle_handlers::delete),
        )
        .route(
            "/vehicles/{vehicle_id}/services",
            get(service_record_handlers::list).post(service_record_handlers::create),
        )
        .route(
            "/vehicles/{vehicle_id}/fuel-logs",
            get(fuel_log_handlers::list).post(fuel_log_handlers::create),
        )
        .route(
            "/vehicles/{vehicle_id}/expenses",
            get(expense_handlers::list).post(expense_handlers::create),
        )
        .route(
            "/vehicles/{vehicle_id}/reminders",
            get(reminder_handlers::list).post(reminder_handlers::create),
        )
        .route(
            "/vehicles/{vehicle_id}/reminders/{id}",
            patch(reminder_handlers::patch),
        )
        .route(
            "/vehicles/{vehicle_id}/documents",
            get(document_handlers::list).post(document_handlers::create),
        )
        .route(
            "/vehicles/{vehicle_id}/summary",
            get(summary_handlers::get_summary),
        )
        .layer(middleware::from_fn_with_state(state.clone(), require_auth));

    Router::new()
        .route("/health", get(health))
        .route("/auth/register", post(auth_handlers::register))
        .route("/auth/login", post(auth_handlers::login))
        .route("/auth/refresh", post(auth_handlers::refresh))
        .route("/auth/logout", post(auth_handlers::logout))
        .merge(protected)
        .with_state(state)
        // Outermost layer: stamp a request id into a task-local + the X-Request-Id
        // header before any handler runs, so success and error envelopes can read it.
        .layer(middleware::from_fn(propagate_request_id))
}
