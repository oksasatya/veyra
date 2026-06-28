use axum::{
    body::Bytes,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    adapters::inbound::http::{
        dto::auth::{
            AuthResponse, AuthTokens, BearerRefreshRequest, LoginRequest, MeResponse,
            RefreshResponse, RegisterRequest, UpdatePreferencesRequest, UserResponse,
        },
        response::ApiResponse,
    },
    application::{
        auth::{
            login::LoginUseCase,
            logout::{LogoutError, LogoutUseCase},
            refresh::{RefreshError, RefreshOutput, RefreshUseCase},
            register::RegisterUseCase,
            AuthOutcome, AuthSession,
        },
        errors::AppError,
        user::update_language::UpdateLanguageUseCase,
    },
    bootstrap::state::AppState,
    domain::user::entity::User,
    ports::repositories::RepositoryError,
};

/// Build the register/login response body directly from the authenticated user
/// returned by the use case — no extra database read.
fn user_response(user: User) -> UserResponse {
    UserResponse {
        id: user.id.to_string(),
        email: user.email.as_str().to_string(),
        name: user.name,
    }
}

/// Build the response body from a freshly-issued session: the user profile plus
/// the access token and the opaque `{family_id}.{raw_secret}` refresh token.
fn auth_response(user: User, session: &AuthSession) -> AuthResponse {
    AuthResponse {
        user: user_response(user),
        tokens: AuthTokens {
            access_token: session.access_token.clone(),
            refresh_token: format!("{}.{}", session.family_id, session.raw_secret),
        },
    }
}

/// POST /auth/register — create a user, open a session, return user + tokens
/// (201). The access token is replayed as `Authorization: Bearer`; the opaque
/// refresh token is stored by the client and replayed to `/auth/refresh`.
pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<Response, AppError> {
    let uc = RegisterUseCase {
        user_repo: state.user_repo.clone(),
        auth: state.auth.clone(),
        sessions: state.sessions.clone(),
        access_ttl_secs: state.access_ttl_secs,
    };
    let AuthOutcome { user, session } = uc.execute(body.email, body.password, body.name).await?;
    Ok(ApiResponse::created(auth_response(user, &session)).into_response())
}

/// POST /auth/login — verify credentials, open a session, return user + tokens
/// (200). Same delivery as [`register`].
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Response, AppError> {
    let uc = LoginUseCase {
        user_repo: state.user_repo.clone(),
        auth: state.auth.clone(),
        sessions: state.sessions.clone(),
        access_ttl_secs: state.access_ttl_secs,
    };
    let AuthOutcome { user, session } = uc.execute(body.email, body.password).await?;
    Ok(ApiResponse::ok(auth_response(user, &session)).into_response())
}

/// POST /auth/refresh — rotate the refresh token and re-issue the auth state.
///
/// Reads the refresh token from the JSON body and returns rotated tokens.
///
/// - Rotated → 200 `{ tokens }`.
/// - Invalid (reuse / not found / missing-or-malformed) → 401.
/// - Unavailable (session store down) → 503.
pub async fn refresh(State(state): State<AppState>, body: Bytes) -> Response {
    let Some((family_id, secret)) = refresh_from_body(&body) else {
        return AppError::Unauthorized.into_response();
    };

    let uc = RefreshUseCase {
        sessions: state.sessions.clone(),
        auth: state.auth.clone(),
        access_ttl_secs: state.access_ttl_secs,
    };

    match uc.execute(family_id, &secret).await {
        Ok(output) => refresh_success(output),
        Err(RefreshError::Invalid) => AppError::Unauthorized.into_response(),
        Err(RefreshError::Unavailable) => AppError::Unavailable.into_response(),
    }
}

/// Build the success body for a rotated session: the new access token and the
/// rotated opaque `{family_id}.{raw_secret}` refresh token.
fn refresh_success(output: RefreshOutput) -> Response {
    let tokens = AuthTokens {
        access_token: output.access_token,
        refresh_token: format!("{}.{}", output.family_id, output.raw_secret),
    };
    ApiResponse::ok(RefreshResponse { tokens }).into_response()
}

/// POST /auth/logout — revoke the session family and its access `sid`.
///
/// The `family_id` is derived from the **refresh** token (`sid == family_id`),
/// NOT the access token — so logout works once the access token has expired.
/// Reads the refresh token from the JSON body.
///
/// - Revocable → 204.
/// - No coherent refresh token → 204 (idempotent; nothing to revoke).
/// - Unavailable (fail-closed Redis error) → 503.
pub async fn logout(State(state): State<AppState>, body: Bytes) -> Response {
    let Some((family_id, _)) = refresh_from_body(&body) else {
        return StatusCode::NO_CONTENT.into_response();
    };

    let uc = LogoutUseCase {
        sessions: state.sessions.clone(),
        access_ttl_secs: state.access_ttl_secs,
    };

    // sid == family_id by invariant — revoke both the family and the access sid.
    match uc.execute(family_id, family_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(LogoutError::Unavailable) => AppError::Unavailable.into_response(),
    }
}

/// Parse a refresh value `{family_id}.{raw_secret}` (split on the FIRST `.` —
/// the base64url secret never contains a `.`).
fn parse_refresh_value(value: &str) -> Option<(Uuid, String)> {
    let (family_part, secret) = value.split_once('.')?;
    let family_id = family_part.parse::<Uuid>().ok()?;
    if secret.is_empty() {
        return None;
    }
    Some((family_id, secret.to_owned()))
}

/// Read + parse the refresh token from the JSON body
/// `{ "refresh_token": "{family_id}.{raw_secret}" }`.
fn refresh_from_body(body: &Bytes) -> Option<(Uuid, String)> {
    let parsed: BearerRefreshRequest = serde_json::from_slice(body).ok()?;
    parse_refresh_value(&parsed.refresh_token)
}

/// Build the `GET /me` / `PATCH /me` body from the authenticated user.
fn me_response(user: User) -> MeResponse {
    MeResponse {
        id: user.id.to_string(),
        email: user.email.as_str().to_string(),
        name: user.name,
        preferred_language: user.preferred_language.as_str().to_string(),
    }
}

/// GET /me — returns the authenticated user's profile.
/// `user_id` is injected by the auth middleware via `Extension<Uuid>`.
pub async fn me(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
) -> Result<ApiResponse<MeResponse>, AppError> {
    let user = state
        .user_repo
        .find_by_id(user_id)
        .await
        .map_err(|e| match e {
            RepositoryError::NotFound => AppError::NotFound,
            other => AppError::from(other),
        })?;

    Ok(ApiResponse::ok(me_response(user)))
}

/// PATCH /me — update the authenticated user's preferred language.
/// Returns 422 (`INVALID_LANGUAGE`) for an unsupported code.
pub async fn patch_me(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Json(body): Json<UpdatePreferencesRequest>,
) -> Result<ApiResponse<MeResponse>, AppError> {
    let uc = UpdateLanguageUseCase {
        user_repo: state.user_repo.clone(),
    };
    let user = uc.execute(user_id, &body.preferred_language).await?;
    Ok(ApiResponse::ok(me_response(user)))
}
