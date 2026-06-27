use axum::{
    middleware,
    routing::{get, patch, post},
    Router,
};

use crate::bootstrap::state::AppState;

use super::{
    handlers::{
        auth as auth_handlers, expenses as expense_handlers, fuel_logs as fuel_log_handlers,
        health::health, reminders as reminder_handlers, service_records as service_record_handlers,
        vehicles as vehicle_handlers,
    },
    middleware::auth::require_auth,
};

/// Builds the axum router. Protected routes sit behind the `require_auth`
/// middleware which validates the Bearer JWT and injects the `Uuid` user-id
/// extension. The state is attached once at the outer router so every handler
/// and middleware receives it via `State<AppState>`.
pub fn build(state: AppState) -> Router {
    let protected = Router::new()
        .route("/me", get(auth_handlers::me))
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
        .layer(middleware::from_fn_with_state(state.clone(), require_auth));

    Router::new()
        .route("/health", get(health))
        .route("/auth/register", post(auth_handlers::register))
        .route("/auth/login", post(auth_handlers::login))
        .merge(protected)
        .with_state(state)
}
