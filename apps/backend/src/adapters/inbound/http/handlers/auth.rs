use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Extension, Json,
};
use axum_extra::extract::cookie::CookieJar;
use uuid::Uuid;

use crate::{
    adapters::inbound::http::{
        auth_mode::wants_bearer,
        cookies::{
            access_cookie, clear, csrf_cookie, random_token, refresh_cookie, refresh_name,
            CookieKind, CookiePolicy,
        },
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

/// Build the access + refresh + csrf cookie jar for a freshly-issued session.
///
/// The refresh cookie value is the opaque `{family_id}.{raw_secret}`; the csrf
/// cookie carries a fresh random token (double-submit pattern).
fn session_cookies(policy: &CookiePolicy, session: &AuthSession) -> CookieJar {
    let refresh_value = format!("{}.{}", session.family_id, session.raw_secret);
    CookieJar::new()
        .add(access_cookie(policy, &session.access_token))
        .add(refresh_cookie(policy, &refresh_value))
        .add(csrf_cookie(policy, &random_token()))
}

/// Build a jar that clears all three auth cookies (used on 401 / logout).
fn clearing_cookies(policy: &CookiePolicy) -> CookieJar {
    CookieJar::new()
        .add(clear(policy, CookieKind::Access))
        .add(clear(policy, CookieKind::Refresh))
        .add(clear(policy, CookieKind::Csrf))
}

/// Build the register/login response body directly from the authenticated user
/// returned by the use case — no extra database read.
fn user_response(user: User) -> UserResponse {
    UserResponse {
        id: user.id.to_string(),
        email: user.email.as_str().to_string(),
        name: user.name,
    }
}

/// Build the bearer-mode body from a freshly-issued session: the user profile
/// plus the access token and the opaque `{family_id}.{raw_secret}` refresh token.
fn bearer_response(user: User, session: &AuthSession) -> AuthResponse {
    AuthResponse {
        user: user_response(user),
        tokens: AuthTokens {
            access_token: session.access_token.clone(),
            refresh_token: format!("{}.{}", session.family_id, session.raw_secret),
        },
    }
}

/// POST /auth/register — create a user, open a session, deliver the auth state.
///
/// - Default (cookie mode): 201, sets access/refresh/csrf cookies, body =
///   [`UserResponse`].
/// - `X-Auth-Mode: bearer`: 201, no cookies, body = [`AuthResponse`] (user +
///   tokens).
pub async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<RegisterRequest>,
) -> Result<Response, AppError> {
    let uc = RegisterUseCase {
        user_repo: state.user_repo.clone(),
        auth: state.auth.clone(),
        sessions: state.sessions.clone(),
        access_ttl_secs: state.access_ttl_secs,
    };
    let AuthOutcome { user, session } = uc.execute(body.email, body.password, body.name).await?;

    if wants_bearer(&headers) {
        return Ok(ApiResponse::created(bearer_response(user, &session)).into_response());
    }
    let jar = session_cookies(&state.cookie_policy, &session);
    Ok((jar, ApiResponse::created(user_response(user))).into_response())
}

/// POST /auth/login — verify credentials, open a session, deliver the auth state.
///
/// Same dual-mode delivery as [`register`], with a 200 status.
pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<LoginRequest>,
) -> Result<Response, AppError> {
    let uc = LoginUseCase {
        user_repo: state.user_repo.clone(),
        auth: state.auth.clone(),
        sessions: state.sessions.clone(),
        access_ttl_secs: state.access_ttl_secs,
    };
    let AuthOutcome { user, session } = uc.execute(body.email, body.password).await?;

    if wants_bearer(&headers) {
        return Ok(ApiResponse::ok(bearer_response(user, &session)).into_response());
    }
    let jar = session_cookies(&state.cookie_policy, &session);
    Ok((jar, ApiResponse::ok(user_response(user))).into_response())
}

/// POST /auth/refresh — rotate the refresh token and re-issue the auth state.
///
/// Cookie mode reads/writes cookies; bearer mode reads the refresh token from
/// the JSON body and returns rotated tokens in the body (no cookies).
///
/// - Rotated → 200 (cookies, or `{ tokens }`).
/// - Invalid (reuse / not found / missing-or-malformed) → 401 (+ clear in
///   cookie mode).
/// - Unavailable (session store down) → 503 (cookies untouched).
pub async fn refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    body: Bytes,
) -> Response {
    let bearer = wants_bearer(&headers);
    let parsed = if bearer {
        refresh_from_body(&body)
    } else {
        read_refresh(&state.cookie_policy, &jar)
    };

    let Some((family_id, secret)) = parsed else {
        return refresh_reject(&state, bearer);
    };

    let uc = RefreshUseCase {
        sessions: state.sessions.clone(),
        auth: state.auth.clone(),
        access_ttl_secs: state.access_ttl_secs,
    };

    match uc.execute(family_id, &secret).await {
        Ok(output) => refresh_success(&state, bearer, output),
        Err(RefreshError::Invalid) => refresh_reject(&state, bearer),
        Err(RefreshError::Unavailable) => AppError::Unavailable.into_response(),
    }
}

/// Build the success response for a rotated session — bearer body or cookie jar.
fn refresh_success(state: &AppState, bearer: bool, output: RefreshOutput) -> Response {
    let session = AuthSession {
        access_token: output.access_token,
        family_id: output.family_id,
        raw_secret: output.raw_secret,
        sid: output.family_id,
    };
    if bearer {
        let tokens = AuthTokens {
            access_token: session.access_token,
            refresh_token: format!("{}.{}", session.family_id, session.raw_secret),
        };
        return ApiResponse::ok(RefreshResponse { tokens }).into_response();
    }
    let fresh = session_cookies(&state.cookie_policy, &session);
    (StatusCode::OK, fresh).into_response()
}

/// 401 on a refused refresh — bare body in bearer mode, clearing cookies on web.
fn refresh_reject(state: &AppState, bearer: bool) -> Response {
    if bearer {
        return AppError::Unauthorized.into_response();
    }
    // Envelope the 401 (per ADR-0008) while clearing the stale cookies.
    (clearing_cookies(&state.cookie_policy), AppError::Unauthorized).into_response()
}

/// POST /auth/logout — revoke the session family and its access `sid`.
///
/// The `family_id` is derived from the **refresh** token (`sid == family_id`),
/// NOT the access token — so logout works once the access token has expired.
/// Cookie mode reads the refresh cookie + clears cookies; bearer mode reads the
/// refresh token from the JSON body.
///
/// - Revocable → 204 (+ clear in cookie mode).
/// - No coherent refresh token → 204 (idempotent; nothing to revoke).
/// - Unavailable (fail-closed Redis error) → 503; cookies are NOT cleared.
pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    body: Bytes,
) -> Response {
    let bearer = wants_bearer(&headers);
    let parsed = if bearer {
        refresh_from_body(&body)
    } else {
        read_refresh(&state.cookie_policy, &jar)
    };

    let Some((family_id, _)) = parsed else {
        return logout_done(&state, bearer);
    };

    let uc = LogoutUseCase {
        sessions: state.sessions.clone(),
        access_ttl_secs: state.access_ttl_secs,
    };

    // sid == family_id by invariant — revoke both the family and the access sid.
    match uc.execute(family_id, family_id).await {
        Ok(()) => logout_done(&state, bearer),
        Err(LogoutError::Unavailable) => AppError::Unavailable.into_response(),
    }
}

/// 204 on logout — bare in bearer mode, clearing cookies on web.
fn logout_done(state: &AppState, bearer: bool) -> Response {
    if bearer {
        return StatusCode::NO_CONTENT.into_response();
    }
    (
        StatusCode::NO_CONTENT,
        clearing_cookies(&state.cookie_policy),
    )
        .into_response()
}

/// Parse a refresh value `{family_id}.{raw_secret}` (split on the FIRST `.` —
/// the base64url secret never contains a `.`). Shared by the cookie reader and
/// the bearer body reader.
fn parse_refresh_value(value: &str) -> Option<(Uuid, String)> {
    let (family_part, secret) = value.split_once('.')?;
    let family_id = family_part.parse::<Uuid>().ok()?;
    if secret.is_empty() {
        return None;
    }
    Some((family_id, secret.to_owned()))
}

/// Read + parse the refresh cookie value. Returns `None` if absent or malformed.
fn read_refresh(policy: &CookiePolicy, jar: &CookieJar) -> Option<(Uuid, String)> {
    let value = jar.get(&refresh_name(policy))?.value().to_owned();
    parse_refresh_value(&value)
}

/// Read + parse the refresh token from a bearer-mode JSON body
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
