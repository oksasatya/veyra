use axum::{
    middleware,
    routing::{get, post},
    Router,
};

use crate::bootstrap::state::AppState;

use super::{
    handlers::{
        auth as auth_handlers,
        health::health,
        service_records as service_record_handlers,
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
        .layer(middleware::from_fn_with_state(state.clone(), require_auth));

    Router::new()
        .route("/health", get(health))
        .route("/auth/register", post(auth_handlers::register))
        .route("/auth/login", post(auth_handlers::login))
        .merge(protected)
        .with_state(state)
}
