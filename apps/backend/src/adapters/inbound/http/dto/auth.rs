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

/// User profile nested inside the register/login [`AuthResponse`].
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

/// Access + refresh token pair. `access_token` is replayed as
/// `Authorization: Bearer`; `refresh_token` is the opaque `{family_id}.{raw_secret}`
/// string the client stores and replays to `/auth/refresh`.
#[derive(Debug, Serialize)]
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
}

/// Register/login response body — the user profile plus the issued token pair.
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user: UserResponse,
    pub tokens: AuthTokens,
}

/// Refresh response body — the rotated token pair.
#[derive(Debug, Serialize)]
pub struct RefreshResponse {
    pub tokens: AuthTokens,
}

/// Request body carrying the refresh token (`/auth/refresh` + `/auth/logout`).
#[derive(Debug, Deserialize)]
pub struct BearerRefreshRequest {
    pub refresh_token: String,
}
