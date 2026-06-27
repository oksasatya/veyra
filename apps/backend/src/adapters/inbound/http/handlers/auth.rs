use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use axum_extra::extract::cookie::CookieJar;
use uuid::Uuid;

use crate::{
    adapters::inbound::http::{
        cookies::{
            access_cookie, clear, csrf_cookie, random_token, refresh_cookie, refresh_name,
            CookieKind, CookiePolicy,
        },
        dto::auth::{LoginRequest, MeResponse, RegisterRequest, UserResponse},
    },
    application::{
        auth::{
            login::LoginUseCase,
            logout::{LogoutError, LogoutUseCase},
            refresh::{RefreshError, RefreshUseCase},
            register::RegisterUseCase,
            AuthOutcome, AuthSession,
        },
        errors::AppError,
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

/// POST /auth/register — create a user, open a session, set auth cookies.
/// Returns 201 with the user profile (no token in the body).
pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<(StatusCode, CookieJar, Json<UserResponse>), AppError> {
    let uc = RegisterUseCase {
        user_repo: state.user_repo.clone(),
        auth: state.auth.clone(),
        sessions: state.sessions.clone(),
        access_ttl_secs: state.access_ttl_secs,
    };
    let AuthOutcome { user, session } = uc.execute(body.email, body.password, body.name).await?;
    let jar = session_cookies(&state.cookie_policy, &session);
    Ok((StatusCode::CREATED, jar, Json(user_response(user))))
}

/// POST /auth/login — verify credentials, open a session, set auth cookies.
/// Returns 200 with the user profile (no token in the body).
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<(StatusCode, CookieJar, Json<UserResponse>), AppError> {
    let uc = LoginUseCase {
        user_repo: state.user_repo.clone(),
        auth: state.auth.clone(),
        sessions: state.sessions.clone(),
        access_ttl_secs: state.access_ttl_secs,
    };
    let AuthOutcome { user, session } = uc.execute(body.email, body.password).await?;
    let jar = session_cookies(&state.cookie_policy, &session);
    Ok((StatusCode::OK, jar, Json(user_response(user))))
}

/// POST /auth/refresh — rotate the refresh token and re-issue all cookies.
///
/// - Rotated → 200 with fresh access/refresh/csrf cookies.
/// - Invalid (reuse / not found / missing-or-malformed cookie) → 401 + clear.
/// - Unavailable (session store down) → 503 (cookies untouched).
pub async fn refresh(State(state): State<AppState>, jar: CookieJar) -> Response {
    let Some((family_id, secret)) = read_refresh(&state.cookie_policy, &jar) else {
        return (
            StatusCode::UNAUTHORIZED,
            clearing_cookies(&state.cookie_policy),
        )
            .into_response();
    };

    let uc = RefreshUseCase {
        sessions: state.sessions.clone(),
        auth: state.auth.clone(),
        access_ttl_secs: state.access_ttl_secs,
    };

    match uc.execute(family_id, &secret).await {
        Ok(output) => {
            let session = AuthSession {
                access_token: output.access_token,
                family_id: output.family_id,
                raw_secret: output.raw_secret,
                sid: output.family_id,
            };
            let fresh = session_cookies(&state.cookie_policy, &session);
            (StatusCode::OK, fresh).into_response()
        }
        Err(RefreshError::Invalid) => (
            StatusCode::UNAUTHORIZED,
            clearing_cookies(&state.cookie_policy),
        )
            .into_response(),
        Err(RefreshError::Unavailable) => StatusCode::SERVICE_UNAVAILABLE.into_response(),
    }
}

/// POST /auth/logout — revoke the session family and its access `sid`.
///
/// The `family_id` is derived from the **refresh** cookie (`sid == family_id`),
/// NOT the access cookie — so logout works even once the access token has
/// expired (the common case). It does NOT sit behind `require_auth`.
///
/// - Refresh cookie present + revocable → 204, all cookies cleared.
/// - No refresh cookie, or an unparseable one → 204 + clear (idempotent /
///   best-effort: there is nothing valid to revoke).
/// - Unavailable (fail-closed Redis error) → 503; cookies are NOT cleared so the
///   client retries against the still-valid session.
pub async fn logout(State(state): State<AppState>, jar: CookieJar) -> Response {
    let Some((family_id, _)) = read_refresh(&state.cookie_policy, &jar) else {
        // No coherent refresh cookie → nothing to revoke; clear and 204.
        return (
            StatusCode::NO_CONTENT,
            clearing_cookies(&state.cookie_policy),
        )
            .into_response();
    };

    let uc = LogoutUseCase {
        sessions: state.sessions.clone(),
        access_ttl_secs: state.access_ttl_secs,
    };

    // sid == family_id by invariant — revoke both the family and the access sid.
    match uc.execute(family_id, family_id).await {
        Ok(()) => (
            StatusCode::NO_CONTENT,
            clearing_cookies(&state.cookie_policy),
        )
            .into_response(),
        Err(LogoutError::Unavailable) => StatusCode::SERVICE_UNAVAILABLE.into_response(),
    }
}

/// Parse the refresh cookie value `{family_id}.{raw_secret}` (split on the FIRST
/// `.` — the base64url secret never contains a `.`). Returns the parsed family
/// id and the raw secret, or `None` if the cookie is absent or malformed.
fn read_refresh(policy: &CookiePolicy, jar: &CookieJar) -> Option<(Uuid, String)> {
    let value = jar.get(&refresh_name(policy))?.value().to_owned();
    let (family_part, secret) = value.split_once('.')?;
    let family_id = family_part.parse::<Uuid>().ok()?;
    if secret.is_empty() {
        return None;
    }
    Some((family_id, secret.to_owned()))
}

/// GET /me — returns the authenticated user's profile.
/// `user_id` is injected by the auth middleware via `Extension<Uuid>`.
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
