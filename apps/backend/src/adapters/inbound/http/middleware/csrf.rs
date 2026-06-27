use axum::{
    extract::{Request, State},
    http::{Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::CookieJar;

use crate::{
    adapters::inbound::http::cookies::{csrf_name, X_CSRF_TOKEN},
    bootstrap::state::AppState,
};

/// Double-submit CSRF protection for mutating, cookie-authenticated routes.
///
/// Safe methods (GET/HEAD/OPTIONS) pass through untouched. For every other
/// method the `X-CSRF-Token` header MUST be present, non-empty, and exactly
/// equal to the value of the csrf cookie; any mismatch, absence, or empty value
/// is rejected with 403.
///
/// Applied to the protected router (AFTER `require_auth`) AND to the
/// auth-mutation routes `/auth/refresh` + `/auth/logout` (which are NOT behind
/// `require_auth`, so logout/refresh keep working once the access token expires
/// — but are still state-changing, so they MUST present the double-submit
/// token). `/auth/register` + `/auth/login` are exempt: no csrf cookie exists
/// yet at that point.
pub async fn require_csrf(State(state): State<AppState>, req: Request, next: Next) -> Response {
    if matches!(*req.method(), Method::GET | Method::HEAD | Method::OPTIONS) {
        return next.run(req).await;
    }

    let jar = CookieJar::from_headers(req.headers());
    let cookie_token = jar
        .get(&csrf_name(&state.cookie_policy))
        .map(|c| c.value().to_owned());
    let header_token = req
        .headers()
        .get(X_CSRF_TOKEN)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned);

    match (header_token, cookie_token) {
        (Some(h), Some(c)) if !h.is_empty() && h == c => next.run(req).await,
        _ => StatusCode::FORBIDDEN.into_response(),
    }
}
