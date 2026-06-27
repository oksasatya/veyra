use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::bootstrap::state::AppState;

/// Axum middleware that validates a Bearer JWT from the `Authorization` header.
/// On success it inserts a `Uuid` (user_id) into the request extensions so
/// protected handlers can extract it via `Extension<Uuid>`.
pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let user_id: Uuid = state
        .auth
        .verify_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    req.extensions_mut().insert(user_id);
    Ok(next.run(req).await)
}
