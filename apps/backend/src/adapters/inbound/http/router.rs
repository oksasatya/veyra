use axum::{
    http::{header, HeaderName, HeaderValue, Method},
    middleware,
    routing::{get, patch, post},
    Router,
};
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::{adapters::inbound::http::cookies::X_CSRF_TOKEN, bootstrap::state::AppState};

use super::{
    handlers::{
        auth as auth_handlers, documents as document_handlers, expenses as expense_handlers,
        fuel_logs as fuel_log_handlers, health::health, reminders as reminder_handlers,
        service_records as service_record_handlers, summary as summary_handlers,
        vehicles as vehicle_handlers,
    },
    middleware::{auth::require_auth, csrf::require_csrf},
};

/// Build a credentialed CORS layer from the configured allowlist.
///
/// Returns `None` when no origins are configured — the API is then same-origin
/// only (no permissive CORS headers are emitted). When credentials are allowed
/// the origin MUST be an explicit allowlist; a wildcard is both illegal in the
/// browser and forbidden by project policy, so unparseable origins are dropped.
fn cors_layer(origins: &[String]) -> Option<CorsLayer> {
    if origins.is_empty() {
        return None;
    }
    let parsed: Vec<HeaderValue> = origins
        .iter()
        .filter_map(|o| match HeaderValue::from_str(o) {
            Ok(v) => Some(v),
            Err(_) => {
                tracing::warn!(origin = %o, "ignoring un-parseable CORS origin");
                None
            }
        })
        .collect();
    if parsed.is_empty() {
        return None;
    }
    Some(
        CorsLayer::new()
            .allow_origin(AllowOrigin::list(parsed))
            .allow_credentials(true)
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::PATCH,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers([header::CONTENT_TYPE, HeaderName::from_static(X_CSRF_TOKEN)]),
    )
}

/// Builds the axum router with three route groups:
///
/// - **Open** (`/health`, `/auth/register`, `/auth/login`) — no layers. Register
///   and login are CSRF-exempt because no csrf cookie exists yet at that point
///   (chicken-and-egg); the response is what sets it.
/// - **Auth-mutation** (`/auth/refresh`, `/auth/logout`) — `require_csrf` ONLY,
///   NOT `require_auth`: they are state-changing (so double-submit CSRF is
///   mandatory) but must keep working once the access token has expired, so
///   they must NOT depend on a valid access cookie.
/// - **Protected** (`/me`, `/vehicles/*`) — `require_auth` (cookie access-token
///   validation + sid-revocation check, injecting `Uuid`) THEN `require_csrf`
///   (double-submit token on mutating methods).
///
/// CORS is derived from `config.cors_allowed_origins` and applied to the whole
/// service.
pub fn build(state: AppState) -> Router {
    let cors = cors_layer(&state.cors_allowed_origins);

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
        .route(
            "/vehicles/{vehicle_id}/documents",
            get(document_handlers::list).post(document_handlers::create),
        )
        .route(
            "/vehicles/{vehicle_id}/summary",
            get(summary_handlers::get_summary),
        )
        // CSRF runs AFTER auth (outer layer runs first on the request path;
        // from_fn layers wrap inside-out, so listing require_csrf first then
        // require_auth makes auth the outermost — auth resolves before CSRF).
        .layer(middleware::from_fn_with_state(state.clone(), require_csrf))
        .layer(middleware::from_fn_with_state(state.clone(), require_auth));

    // Refresh + logout: state-changing, CSRF-guarded, but NOT auth-guarded
    // (they must run even when the access token has expired). Only `require_csrf`
    // is layered here so the double-submit token is actually checked.
    let auth_mutations = Router::new()
        .route("/auth/refresh", post(auth_handlers::refresh))
        .route("/auth/logout", post(auth_handlers::logout))
        .layer(middleware::from_fn_with_state(state.clone(), require_csrf));

    let mut app = Router::new()
        .route("/health", get(health))
        .route("/auth/register", post(auth_handlers::register))
        .route("/auth/login", post(auth_handlers::login))
        .merge(auth_mutations)
        .merge(protected)
        .with_state(state);

    if let Some(cors) = cors {
        app = app.layer(cors);
    }
    app
}
