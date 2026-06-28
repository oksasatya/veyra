use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, HeaderMap},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::{application::errors::AppError, bootstrap::state::AppState};

/// Extract the token from an `Authorization: Bearer <token>` header, if present.
fn bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get(AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(str::to_owned)
}

/// Axum middleware that authenticates the request via an
/// `Authorization: Bearer <jwt>` header: verify the access token, check
/// sid-revocation (fail-open on a store error), and inject `user_id` into the
/// request extensions. Any failure → a bare `401` envelope.
pub async fn require_auth(State(state): State<AppState>, mut req: Request, next: Next) -> Response {
    let Some(token) = bearer_token(req.headers()) else {
        return AppError::Unauthorized.into_response();
    };

    let claims = match state.auth.verify_access(&token) {
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
