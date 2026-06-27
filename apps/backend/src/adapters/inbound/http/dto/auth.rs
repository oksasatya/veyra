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
}
