use axum::{extract::State, http::StatusCode, Extension, Json};
use uuid::Uuid;

use crate::{
    adapters::inbound::http::dto::auth::{
        LoginRequest, MeResponse, RegisterRequest, TokenResponse,
    },
    application::{
        auth::{login::LoginUseCase, register::RegisterUseCase},
        errors::AppError,
    },
    bootstrap::state::AppState,
    ports::repositories::RepositoryError,
};

/// POST /auth/register — creates a new user and returns a signed JWT.
/// Returns 201 Created on success.
pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<TokenResponse>), AppError> {
    let uc = RegisterUseCase {
        user_repo: state.user_repo.clone(),
        auth: state.auth.clone(),
    };
    let token = uc.execute(body.email, body.password, body.name).await?;
    Ok((StatusCode::CREATED, Json(TokenResponse { token })))
}

/// POST /auth/login — verifies credentials and returns a signed JWT.
/// Returns 200 OK on success.
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<TokenResponse>, AppError> {
    let uc = LoginUseCase {
        user_repo: state.user_repo.clone(),
        auth: state.auth.clone(),
    };
    let token = uc.execute(body.email, body.password).await?;
    Ok(Json(TokenResponse { token }))
}

/// GET /me — returns the authenticated user's profile.
/// Requires a valid Bearer token (injected as `Extension<Uuid>` by the auth middleware).
pub async fn me(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<MeResponse>, AppError> {
    let user = state
        .user_repo
        .find_by_id(user_id)
        .await
        .map_err(|e| match e {
            RepositoryError::NotFound => AppError::NotFound,
            other => AppError::from(other),
        })?;

    Ok(Json(MeResponse {
        id: user.id.to_string(),
        email: user.email.as_str().to_string(),
        name: user.name,
    }))
}
