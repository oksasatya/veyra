use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::CookieJar;

use crate::{
    adapters::inbound::http::cookies::{access_name, clear, CookieKind, CookiePolicy},
    bootstrap::state::AppState,
};

/// Build an `UNAUTHORIZED` response that also clears the access, refresh, and
/// csrf cookies. Used on every auth-rejection path so a stale or invalid
/// session never lingers in the browser jar.
pub fn unauthorized_clearing(policy: &CookiePolicy) -> Response {
    let jar = CookieJar::new()
        .add(clear(policy, CookieKind::Access))
        .add(clear(policy, CookieKind::Refresh))
        .add(clear(policy, CookieKind::Csrf));
    (StatusCode::UNAUTHORIZED, jar).into_response()
}

/// Axum middleware that authenticates the request via the HttpOnly access
/// cookie.
///
/// 1. Read the access cookie (`access_name(policy)`); absent → 401 + clear.
/// 2. `verify_access` the JWT; invalid/expired → 401 + clear.
/// 3. `is_session_revoked(sid)` — **fail-open**: `Ok(true)` → reject (401 +
///    clear); `Ok(false)` → allow; `Err(_)` → log a warning and allow (the read
///    path tolerates a Redis blip rather than locking everyone out).
/// 4. Inject `user_id` into the request extensions for downstream handlers.
pub async fn require_auth(State(state): State<AppState>, mut req: Request, next: Next) -> Response {
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
            // Fail-open: a session-store blip on the READ path must not lock out
            // every user. Log and allow; the write paths (refresh/logout) remain
            // fail-closed.
            tracing::warn!(error = %e, "session revocation check failed; allowing request (fail-open)");
        }
    }

    req.extensions_mut().insert(claims.user_id);
    next.run(req).await
}
