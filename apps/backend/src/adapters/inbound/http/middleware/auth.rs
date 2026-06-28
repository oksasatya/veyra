use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, HeaderMap},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::CookieJar;

use crate::{
    adapters::inbound::http::cookies::{access_name, clear, CookieKind, CookiePolicy},
    application::errors::AppError,
    bootstrap::state::AppState,
};

/// Build an `UNAUTHORIZED` response that also clears the access, refresh, and
/// csrf cookies. Used on every cookie-path auth-rejection so a stale or invalid
/// session never lingers in the browser jar.
pub fn unauthorized_clearing(policy: &CookiePolicy) -> Response {
    let jar = CookieJar::new()
        .add(clear(policy, CookieKind::Access))
        .add(clear(policy, CookieKind::Refresh))
        .add(clear(policy, CookieKind::Csrf));
    // Envelope the 401 (per ADR-0008) while still clearing the stale cookies.
    (jar, AppError::Unauthorized).into_response()
}

/// Extract the token from an `Authorization: Bearer <token>` header, if present.
fn bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get(AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(str::to_owned)
}

/// Axum middleware that authenticates the request.
///
/// 1. **Bearer path (native mobile):** if an `Authorization: Bearer <jwt>` header
///    is present, verify it, check sid-revocation (fail-open), inject `user_id`.
///    401s carry NO cookies.
/// 2. **Cookie path (web, unchanged):** otherwise read the HttpOnly access
///    cookie, verify it, check revocation, inject `user_id`. 401s clear cookies.
pub async fn require_auth(State(state): State<AppState>, mut req: Request, next: Next) -> Response {
    if let Some(token) = bearer_token(req.headers()) {
        return authenticate_bearer(&state, &token, req, next).await;
    }

    let jar = CookieJar::from_headers(req.headers());
    let Some(cookie) = jar.get(&access_name(&state.cookie_policy)) else {
        return unauthorized_clearing(&state.cookie_policy);
    };

    let claims = match state.auth.verify_access(cookie.value()) {
        Ok(claims) => claims,
        Err(_) => return unauthorized_clearing(&state.cookie_policy),
    };

    match state.sessions.is_session_revoked(claims.sid).await {
        Ok(true) => return unauthorized_clearing(&state.cookie_policy),
        Ok(false) => {}
        Err(e) => {
            tracing::warn!(error = %e, "session revocation check failed; allowing request (fail-open)");
        }
    }

    req.extensions_mut().insert(claims.user_id);
    next.run(req).await
}

/// Bearer-path authentication: identical session logic to the cookie path, just
/// a different token source and bare (no-cookie) 401s.
async fn authenticate_bearer(
    state: &AppState,
    token: &str,
    mut req: Request,
    next: Next,
) -> Response {
    let claims = match state.auth.verify_access(token) {
        Ok(claims) => claims,
        Err(_) => return AppError::Unauthorized.into_response(),
    };

    match state.sessions.is_session_revoked(claims.sid).await {
        Ok(true) => return AppError::Unauthorized.into_response(),
        Ok(false) => {}
        Err(e) => {
            tracing::warn!(error = %e, "session revocation check failed; allowing request (fail-open)");
        }
    }

    req.extensions_mut().insert(claims.user_id);
    next.run(req).await
}
