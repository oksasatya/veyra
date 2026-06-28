use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// User profile returned by register/login (cookies carry the auth state;
/// there is intentionally no token in the body) and by `GET /me`.
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub id: String,
    pub email: String,
    pub name: String,
    /// The user's preferred language code (`"en"` / `"id"`) for server-generated
    /// content. The client localizes its own UI independently.
    pub preferred_language: String,
}

/// Body for `PATCH /me` — updates the user's preferred language.
#[derive(Debug, Deserialize)]
pub struct UpdatePreferencesRequest {
    pub preferred_language: String,
}

/// Access + refresh pair returned only in bearer mode. `refresh_token` is the
/// opaque `{family_id}.{raw_secret}` string the client stores and replays.
#[derive(Debug, Serialize)]
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
}

/// Bearer-mode register/login body. Cookie mode keeps returning bare
/// [`UserResponse`].
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user: UserResponse,
    pub tokens: AuthTokens,
}

/// Bearer-mode refresh body.
#[derive(Debug, Serialize)]
pub struct RefreshResponse {
    pub tokens: AuthTokens,
}

/// Bearer-mode request body carrying the refresh token (refresh + logout).
#[derive(Debug, Deserialize)]
pub struct BearerRefreshRequest {
    pub refresh_token: String,
}
