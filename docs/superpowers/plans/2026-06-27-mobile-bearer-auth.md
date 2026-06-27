# Dual-Mode Auth (Bearer for Mobile) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> **Every Rust task:** Invoke `rust-expert` first (hub — auto-chains the Rust family + algorithmic-complexity + superpowers). Apply the Rust quality-gate block below (NOT Sonar). Honor the per-task `TDD:` verdict.

**Goal:** Add an opt-in `Authorization: Bearer` delivery path for native mobile clients to Veyra's auth, layered on top of the existing cookie/CSRF flow without changing the session model.

**Architecture:** One session core (access JWT + rotating Redis refresh family + sid revocation) serves two delivery modes. A request header `X-Auth-Mode: bearer` selects bearer mode at the four auth endpoints; protected routes are disambiguated by the presence of an `Authorization` header. Only token *delivery* (response body vs cookies) and *extraction* (Authorization header / JSON body vs cookies) differ between modes.

**Tech Stack:** Rust 2021 · axum 0.8 · sqlx 0.8 · fred (Redis) · jsonwebtoken · argon2 · axum-test 16 · testcontainers (Postgres + Redis) · cargo nextest

**Spec:** `docs/superpowers/specs/2026-06-27-mobile-bearer-auth-design.md` · **ADR:** ADR-0007 (extends ADR-0006)

## Global Constraints

- Edition 2021; `#![forbid(unsafe_code)]` crate-wide.
- **Strictly additive — the cookie/CSRF flow (web) MUST remain byte-for-byte identical when no `X-Auth-Mode: bearer` header is sent.** The existing cookie-mode test suite is the regression guard.
- Bearer mode is gated by `X-Auth-Mode: bearer` (case-insensitive) at register/login/refresh/logout.
- `require_csrf` is bypassed when the request carries an `Authorization` header **OR** `X-Auth-Mode: bearer` (bearer/native = no cookie = no CSRF surface).
- `require_auth` authenticates by `Authorization: Bearer <jwt>` first; falls back to the access cookie.
- Session model unchanged: reuse `AuthPort::verify_access`, `SessionStore::rotate/revoke/is_session_revoked`, `RefreshUseCase`, `AuthOutcome`/`AuthSession`/`RefreshOutput`. Invariant: `sid == family_id`.
- Refresh token wire format unchanged: opaque `{family_id}.{raw_secret}` (split on the FIRST `.`).
- Error body shape (actual, per `AppError::into_response`): flat `{ "error": "<message>" }`. Bearer 401s reuse `AppError::Unauthorized.into_response()`; cookie 401s keep `unauthorized_clearing` (no body, clears cookies).
- DRY: ONE refresh-value parse helper (`parse_refresh_value`) used by both cookie and body extraction. ONE `wants_bearer` helper used by handlers and the CSRF middleware.

### Rust quality gate — write compliant from the first commit (NO Sonar; clippy is the gate)

- `clippy::too_many_arguments` — ≤7 params (aim ≤5); past that, a params struct.
- `clippy::cognitive_complexity` — ≤15; extract named helpers (project rule: aim ≤20 lines/fn, hard cap 40).
- NO `.unwrap()` / `.expect()` / `panic!` / `todo!()` on production paths — return `Result` + `?`. (Tests MAY unwrap/expect freely.)
- Duplicated string literal ≥3× → a module-level `const`.
- `#![forbid(unsafe_code)]`.
- Errors: `thiserror` in domain/application; map to HTTP at the one `IntoResponse` choke point; never `let _ = fallible();` on a path whose failure matters.
- Async: never hold a `std::sync::Mutex` guard across `.await`; never block the runtime.
- sqlx: parameterized only (unchanged here — no new queries).
- Verify before "done": `cargo fmt --check → cargo clippy --all-targets --all-features -- -D warnings → cargo nextest run`.

When fixing one instance of a rule, scan sibling files for the same shape and fix-forward. When reviewing, check the diff against this list BEFORE marking compliant.

---

## File Structure

```
apps/backend/src/adapters/inbound/http/
├── auth_mode.rs        ← NEW: X-Auth-Mode header constant + wants_bearer() (+ unit test)
├── mod.rs              ← MODIFY: pub mod auth_mode;
├── dto/auth.rs         ← MODIFY: add AuthTokens, AuthResponse, RefreshResponse, BearerRefreshRequest
├── handlers/auth.rs    ← MODIFY: register/login/refresh/logout branch on mode; extract parse_refresh_value
├── middleware/auth.rs  ← MODIFY: require_auth bearer branch + bearer helpers
└── middleware/csrf.rs  ← MODIFY: require_csrf bearer/native bypass
packages/openapi/veyra.yaml          ← MODIFY: X-Auth-Mode param + bearer schemas + bearer security scheme
apps/backend/tests/common/mod.rs     ← MODIFY: bearer test helpers (register_and_login_bearer, header builders)
apps/backend/tests/auth_test.rs      ← MODIFY: bearer-mode integration cases (beside cookie cases)
```

No changes to: application use cases, ports, domain, Redis session store/cache, router wiring (same middlewares; their internal branching changes).

---

### Task 1: `X-Auth-Mode` helper + bearer DTOs

**TDD: yes** (for `wants_bearer` — a pure predicate with a clear contract). DTOs are plain serde structs (no logic), verified by the compiler + downstream task tests.

**Files:**
- Create: `apps/backend/src/adapters/inbound/http/auth_mode.rs`
- Modify: `apps/backend/src/adapters/inbound/http/mod.rs`
- Modify: `apps/backend/src/adapters/inbound/http/dto/auth.rs`

**Interfaces:**
- Produces: `pub const AUTH_MODE_HEADER: &str = "x-auth-mode";`
- Produces: `pub fn wants_bearer(headers: &axum::http::HeaderMap) -> bool`
- Produces: DTOs `AuthTokens { access_token, refresh_token }`, `AuthResponse { user, tokens }`, `RefreshResponse { tokens }`, `BearerRefreshRequest { refresh_token }`

- [ ] **Step 1: Write the failing test for `wants_bearer`**

Create `apps/backend/src/adapters/inbound/http/auth_mode.rs`:

```rust
//! `X-Auth-Mode` request-header detection. Native (mobile) clients opt into the
//! Bearer delivery path by sending `X-Auth-Mode: bearer`; everything else is the
//! default cookie flow.

use axum::http::HeaderMap;

/// Request header that opts a client into bearer-token delivery.
pub const AUTH_MODE_HEADER: &str = "x-auth-mode";

const AUTH_MODE_BEARER: &str = "bearer";

/// True when the request asks for bearer-mode delivery (`X-Auth-Mode: bearer`,
/// case-insensitive). Absent or any other value → false (cookie mode).
pub fn wants_bearer(headers: &HeaderMap) -> bool {
    headers
        .get(AUTH_MODE_HEADER)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.eq_ignore_ascii_case(AUTH_MODE_BEARER))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderMap, HeaderValue};

    fn headers_with(mode: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(AUTH_MODE_HEADER, HeaderValue::from_str(mode).unwrap());
        h
    }

    #[test]
    fn bearer_value_detected() {
        assert!(wants_bearer(&headers_with("bearer")));
    }

    #[test]
    fn bearer_value_case_insensitive() {
        assert!(wants_bearer(&headers_with("Bearer")));
    }

    #[test]
    fn missing_header_is_cookie_mode() {
        assert!(!wants_bearer(&HeaderMap::new()));
    }

    #[test]
    fn other_value_is_cookie_mode() {
        assert!(!wants_bearer(&headers_with("cookie")));
    }
}
```

- [ ] **Step 2: Register the module — edit `apps/backend/src/adapters/inbound/http/mod.rs`**

Add `pub mod auth_mode;` alongside the existing `pub mod` declarations (keep alphabetical if the file is ordered).

- [ ] **Step 3: Run the test to verify it passes**

Run: `cd apps/backend && cargo nextest run adapters::inbound::http::auth_mode`
Expected: 4 tests pass.

- [ ] **Step 4: Add the bearer DTOs — edit `apps/backend/src/adapters/inbound/http/dto/auth.rs`**

Append:

```rust
/// Access + refresh pair returned only in bearer mode. `refresh_token` is the
/// opaque `{family_id}.{raw_secret}` string the client stores and replays.
#[derive(Debug, Serialize)]
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
}

/// Bearer-mode register/login body. Cookie mode keeps returning bare `UserResponse`.
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
```

- [ ] **Step 5: Verify it compiles**

Run: `cd apps/backend && cargo clippy --all-targets --all-features -- -D warnings`
Expected: clean (some DTOs unused until later tasks — that is fine; they are `pub`).

- [ ] **Step 6: Commit**

```bash
git add apps/backend/src/adapters/inbound/http/auth_mode.rs \
        apps/backend/src/adapters/inbound/http/mod.rs \
        apps/backend/src/adapters/inbound/http/dto/auth.rs
git commit -m "feat(auth): add X-Auth-Mode detection and bearer-mode DTOs"
```

---

### Task 2: Bearer mode for register + login

**TDD: yes** — clear input→output contract and a security invariant (no Set-Cookie in bearer mode).

**Files:**
- Modify: `apps/backend/src/adapters/inbound/http/handlers/auth.rs`
- Modify: `apps/backend/tests/common/mod.rs`
- Modify: `apps/backend/tests/auth_test.rs`

**Interfaces:**
- Consumes: `wants_bearer`, `AuthTokens`, `AuthResponse`, `AuthOutcome { user, session }`, `AuthSession { access_token, family_id, raw_secret, sid }`.
- Produces: bearer test helpers `register_and_login_bearer(&TestApp, &str) -> (String, String)` (access, refresh), `auth_mode_header()`, `bearer_header(&str)`.

- [ ] **Step 1: Add bearer test helpers — edit `apps/backend/tests/common/mod.rs`**

Add (near `csrf_header`):

```rust
use axum::http::header::AUTHORIZATION;

/// The `X-Auth-Mode: bearer` header pair. Attach to register/login/refresh/logout
/// requests that should use bearer-token delivery.
#[allow(dead_code)]
pub fn auth_mode_header() -> (HeaderName, HeaderValue) {
    (
        HeaderName::from_static("x-auth-mode"),
        HeaderValue::from_static("bearer"),
    )
}

/// The `Authorization: Bearer <access>` header pair for protected requests.
#[allow(dead_code)]
pub fn bearer_header(access: &str) -> (HeaderName, HeaderValue) {
    (
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {access}")).expect("valid bearer header"),
    )
}

/// Register + login a user in BEARER mode; returns `(access_token, refresh_token)`
/// read from the JSON body. No cookies are set in bearer mode.
#[allow(dead_code)]
pub async fn register_and_login_bearer(app: &TestApp, email: &str) -> (String, String) {
    let (n, v) = auth_mode_header();
    app.client
        .post("/auth/register")
        .add_header(n.clone(), v.clone())
        .json(&json!({ "email": email, "password": "password123", "name": "User" }))
        .await;
    let resp = app
        .client
        .post("/auth/login")
        .add_header(n, v)
        .json(&json!({ "email": email, "password": "password123" }))
        .await;
    let body: serde_json::Value = resp.json();
    let access = body["tokens"]["access_token"].as_str().unwrap().to_string();
    let refresh = body["tokens"]["refresh_token"].as_str().unwrap().to_string();
    (access, refresh)
}
```

- [ ] **Step 2: Write the failing integration test — edit `apps/backend/tests/auth_test.rs`**

```rust
#[tokio::test]
async fn bearer_login_returns_tokens_and_no_cookies() {
    let app = common::spawn_app().await;
    let (n, v) = common::auth_mode_header();

    app.client
        .post("/auth/register")
        .add_header(n.clone(), v.clone())
        .json(&serde_json::json!({ "email": "b@e.com", "password": "password123", "name": "U" }))
        .await;

    let resp = app
        .client
        .post("/auth/login")
        .add_header(n, v)
        .json(&serde_json::json!({ "email": "b@e.com", "password": "password123" }))
        .await;

    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert!(body["tokens"]["access_token"].is_string());
    assert!(body["tokens"]["refresh_token"].is_string());
    // Security invariant: bearer mode sets NO cookies.
    assert!(resp.headers().get_all("set-cookie").iter().next().is_none());
}
```

- [ ] **Step 3: Run it to verify it fails**

Run: `cd apps/backend && cargo nextest run --test auth_test bearer_login_returns_tokens_and_no_cookies`
Expected: FAIL — login still returns `UserResponse` with cookies, no `tokens` in body. (Requires Docker.)

- [ ] **Step 4: Implement the bearer branch — edit `apps/backend/src/adapters/inbound/http/handlers/auth.rs`**

Add imports:

```rust
use axum::{http::HeaderMap, response::IntoResponse};

use crate::adapters::inbound::http::{
    auth_mode::wants_bearer,
    dto::auth::{AuthResponse, AuthTokens, /* existing */ },
};
```

Add a helper (keeps the handlers small):

```rust
/// Build the bearer-mode body from a freshly-issued session: user profile + the
/// access token and the opaque `{family_id}.{raw_secret}` refresh token.
fn bearer_response(user: User, session: &AuthSession) -> AuthResponse {
    AuthResponse {
        user: user_response(user),
        tokens: AuthTokens {
            access_token: session.access_token.clone(),
            refresh_token: format!("{}.{}", session.family_id, session.raw_secret),
        },
    }
}
```

Change `register` and `login` to take `HeaderMap` and return `Result<Response, AppError>`, branching on mode. `register`:

```rust
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
        return Ok((StatusCode::CREATED, Json(bearer_response(user, &session))).into_response());
    }
    let jar = session_cookies(&state.cookie_policy, &session);
    Ok((StatusCode::CREATED, jar, Json(user_response(user))).into_response())
}
```

`login` mirrors this with `StatusCode::OK` and `LoginUseCase` (same shape — but write it out fully, do not abbreviate):

```rust
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
        return Ok((StatusCode::OK, Json(bearer_response(user, &session))).into_response());
    }
    let jar = session_cookies(&state.cookie_policy, &session);
    Ok((StatusCode::OK, jar, Json(user_response(user))).into_response())
}
```

Add `use axum::response::Response;` if not already imported.

- [ ] **Step 5: Run the bearer test + the existing cookie register/login tests**

Run: `cd apps/backend && cargo nextest run --test auth_test`
Expected: the new bearer test passes; all existing cookie register/login tests still pass (no header → identical behavior).

- [ ] **Step 6: Verify gate + commit**

```bash
cd apps/backend && cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings
git add apps/backend/src/adapters/inbound/http/handlers/auth.rs apps/backend/tests/
git commit -m "feat(auth): bearer-mode register/login (tokens in body, no cookies)"
```

---

### Task 3: `require_auth` bearer branch + `require_csrf` bypass

**TDD: yes** — protected-route access control with security invariants, exercised end-to-end via the router.

**Files:**
- Modify: `apps/backend/src/adapters/inbound/http/middleware/csrf.rs`
- Modify: `apps/backend/src/adapters/inbound/http/middleware/auth.rs`
- Modify: `apps/backend/tests/auth_test.rs`

**Interfaces:**
- Consumes: `wants_bearer`, `AuthPort::verify_access`, `SessionStore::is_session_revoked`, `AppError::Unauthorized`.

- [ ] **Step 1: Write failing integration tests — edit `apps/backend/tests/auth_test.rs`**

```rust
#[tokio::test]
async fn bearer_protected_route_succeeds() {
    let app = common::spawn_app().await;
    let (access, _refresh) = common::register_and_login_bearer(&app, "p@e.com").await;
    let (n, v) = common::bearer_header(&access);

    let resp = app.client.get("/me").add_header(n, v).await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["email"], "p@e.com");
}

#[tokio::test]
async fn bearer_mutation_needs_no_csrf_header() {
    let app = common::spawn_app().await;
    let (access, _refresh) = common::register_and_login_bearer(&app, "m@e.com").await;
    let (n, v) = common::bearer_header(&access);

    // POST a vehicle with Bearer auth and NO X-CSRF-Token header → must succeed.
    let resp = app
        .client
        .post("/vehicles")
        .add_header(n, v)
        .json(&serde_json::json!({
            "brand": "Toyota", "model": "Avanza", "year": 2020,
            "plate_number": "B 1 ABC", "fuel_type": "petrol", "current_odometer": 1000
        }))
        .await;
    assert_eq!(resp.status_code(), 201);
}

#[tokio::test]
async fn bearer_invalid_token_is_unauthorized_without_cookies() {
    let app = common::spawn_app().await;
    let (n, v) = common::bearer_header("not.a.real.jwt");

    let resp = app.client.get("/me").add_header(n, v).await;
    assert_eq!(resp.status_code(), 401);
    assert!(resp.headers().get_all("set-cookie").iter().next().is_none());
}
```

> If the vehicle create DTO field names differ, copy them from `dto/vehicle.rs`; the create test only needs a valid body.

- [ ] **Step 2: Run to verify they fail**

Run: `cd apps/backend && cargo nextest run --test auth_test bearer_`
Expected: FAIL — `require_csrf` 403s the bearer mutation, `require_auth` ignores the Authorization header (cookie-only).

- [ ] **Step 3: Add the CSRF bypass — edit `apps/backend/src/adapters/inbound/http/middleware/csrf.rs`**

Add imports and a bypass immediately after the safe-method check:

```rust
use axum::http::header::AUTHORIZATION;

use crate::adapters::inbound::http::auth_mode::wants_bearer;
```

Inside `require_csrf`, after the `GET | HEAD | OPTIONS` early return:

```rust
    // Bearer / native requests are not cookie-authenticated → no CSRF surface.
    // Refresh/logout in bearer mode carry no Authorization header (the access
    // token has expired), so X-Auth-Mode is the discriminator they rely on.
    if req.headers().contains_key(AUTHORIZATION) || wants_bearer(req.headers()) {
        return next.run(req).await;
    }
```

- [ ] **Step 4: Add the bearer branch — edit `apps/backend/src/adapters/inbound/http/middleware/auth.rs`**

Add imports:

```rust
use axum::http::{header::AUTHORIZATION, HeaderMap};

use crate::application::errors::AppError;
```

Add helpers and a bearer branch at the top of `require_auth`:

```rust
/// Extract the token from an `Authorization: Bearer <token>` header, if present.
fn bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get(AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(str::to_owned)
}

/// Authenticate a bearer (native) request: verify the access JWT, honor sid
/// revocation (fail-open on a store blip, like the cookie path), inject `user_id`.
/// 401s carry NO clearing cookies — there are none on the bearer path.
async fn authenticate_bearer(
    state: &AppState,
    token: &str,
    mut req: Request,
    next: Next,
) -> Response {
    let claims = match state.auth.verify_access(token) {
        Ok(c) => c,
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
```

Then make `require_auth` try bearer first:

```rust
pub async fn require_auth(State(state): State<AppState>, req: Request, next: Next) -> Response {
    // Bearer path (native mobile) takes precedence when an Authorization header is present.
    if let Some(token) = bearer_token(req.headers()) {
        return authenticate_bearer(&state, &token, req, next).await;
    }

    // Cookie path (web) — unchanged.
    let jar = CookieJar::from_headers(req.headers());
    // ... existing cookie logic unchanged ...
}
```

> Keep the existing cookie body exactly as-is below the bearer branch. `require_auth`'s `req` is no longer `mut` at the top level (the bearer branch moves it into `authenticate_bearer`; the cookie branch re-derives the jar from headers as before, then needs its own `mut req` — keep the existing `mut req: Request` by shadowing inside the cookie path, or leave the signature `mut req` and pass `req` along. Pick whichever keeps clippy clean.)

`AppError::Unauthorized.into_response()` yields `401 {"error":"unauthorized"}` — DRY with the single error choke point; no duplicated literal.

- [ ] **Step 5: Run the bearer tests + full cookie suite**

Run: `cd apps/backend && cargo nextest run --test auth_test`
Expected: all bearer tests pass; every existing cookie test (CSRF enforced, cookie auth) still passes.

- [ ] **Step 6: Verify gate + commit**

```bash
cd apps/backend && cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings
git add apps/backend/src/adapters/inbound/http/middleware/ apps/backend/tests/auth_test.rs
git commit -m "feat(auth): bearer require_auth branch + CSRF bypass for native requests"
```

---

### Task 4: Bearer mode for refresh + logout

**TDD: yes** — rotation, reuse-detection, and revocation are security-critical contracts.

**Files:**
- Modify: `apps/backend/src/adapters/inbound/http/handlers/auth.rs`
- Modify: `apps/backend/tests/auth_test.rs`

**Interfaces:**
- Consumes: `wants_bearer`, `BearerRefreshRequest`, `RefreshResponse`, `AuthTokens`, `RefreshUseCase`/`RefreshOutput`/`RefreshError`, `LogoutUseCase`/`LogoutError`, `AppError::Unauthorized`.
- Produces: `fn parse_refresh_value(value: &str) -> Option<(Uuid, String)>` (shared by cookie + body).

- [ ] **Step 1: Extract the shared refresh-value parser — edit `handlers/auth.rs`**

Refactor the existing `read_refresh` to delegate to a value-only parser (DRY — the body path reuses it):

```rust
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

fn read_refresh(policy: &CookiePolicy, jar: &CookieJar) -> Option<(Uuid, String)> {
    let value = jar.get(&refresh_name(policy))?.value().to_owned();
    parse_refresh_value(&value)
}
```

- [ ] **Step 2: Write the failing bearer refresh/logout tests — edit `tests/auth_test.rs`**

```rust
#[tokio::test]
async fn bearer_refresh_rotates_and_returns_new_tokens() {
    let app = common::spawn_app().await;
    let (_access, refresh) = common::register_and_login_bearer(&app, "r@e.com").await;
    let (n, v) = common::auth_mode_header();

    let resp = app
        .client
        .post("/auth/refresh")
        .add_header(n, v)
        .json(&serde_json::json!({ "refresh_token": refresh }))
        .await;

    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert!(body["tokens"]["access_token"].is_string());
    assert!(body["tokens"]["refresh_token"].is_string());
}

#[tokio::test]
async fn bearer_refresh_reuse_is_rejected() {
    // grace = 0 → the old secret is immediately invalid after one rotation.
    let app = common::spawn_app_with_grace(0).await;
    let (_a, refresh) = common::register_and_login_bearer(&app, "reuse@e.com").await;
    let (n, v) = common::auth_mode_header();

    // First refresh rotates successfully.
    app.client.post("/auth/refresh").add_header(n.clone(), v.clone())
        .json(&serde_json::json!({ "refresh_token": refresh })).await
        .assert_status_ok();

    // Replaying the now-rotated token → 401.
    let resp = app.client.post("/auth/refresh").add_header(n, v)
        .json(&serde_json::json!({ "refresh_token": refresh })).await;
    assert_eq!(resp.status_code(), 401);
}

#[tokio::test]
async fn bearer_logout_revokes_then_refresh_fails() {
    let app = common::spawn_app().await;
    let (_a, refresh) = common::register_and_login_bearer(&app, "lo@e.com").await;
    let (n, v) = common::auth_mode_header();

    app.client.post("/auth/logout").add_header(n.clone(), v.clone())
        .json(&serde_json::json!({ "refresh_token": refresh })).await
        .assert_status(axum::http::StatusCode::NO_CONTENT);

    let resp = app.client.post("/auth/refresh").add_header(n, v)
        .json(&serde_json::json!({ "refresh_token": refresh })).await;
    assert_eq!(resp.status_code(), 401);
}
```

- [ ] **Step 3: Run to verify they fail**

Run: `cd apps/backend && cargo nextest run --test auth_test bearer_refresh bearer_logout`
Expected: FAIL — refresh/logout read only the cookie; bearer body is ignored.

- [ ] **Step 4: Add bearer branches to `refresh` and `logout` — edit `handlers/auth.rs`**

Add imports: `axum::{body::Bytes, http::HeaderMap}` and `dto::auth::{BearerRefreshRequest, RefreshResponse, AuthTokens}`.

Add a small body-extraction helper:

```rust
/// Parse the refresh token out of a bearer-mode JSON body `{ "refresh_token": "..." }`.
fn refresh_from_body(body: &Bytes) -> Option<(Uuid, String)> {
    let parsed: BearerRefreshRequest = serde_json::from_slice(body).ok()?;
    parse_refresh_value(&parsed.refresh_token)
}
```

Change `refresh` to accept headers + body and branch. Bearer path returns tokens in the body, sets no cookies:

```rust
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
        return if bearer {
            AppError::Unauthorized.into_response()
        } else {
            (StatusCode::UNAUTHORIZED, clearing_cookies(&state.cookie_policy)).into_response()
        };
    };

    let uc = RefreshUseCase {
        sessions: state.sessions.clone(),
        auth: state.auth.clone(),
        access_ttl_secs: state.access_ttl_secs,
    };

    match uc.execute(family_id, &secret).await {
        Ok(output) => refresh_success(&state, bearer, output),
        Err(RefreshError::Invalid) => {
            if bearer {
                AppError::Unauthorized.into_response()
            } else {
                (StatusCode::UNAUTHORIZED, clearing_cookies(&state.cookie_policy)).into_response()
            }
        }
        Err(RefreshError::Unavailable) => StatusCode::SERVICE_UNAVAILABLE.into_response(),
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
        (StatusCode::OK, Json(RefreshResponse { tokens })).into_response()
    } else {
        (StatusCode::OK, session_cookies(&state.cookie_policy, &session)).into_response()
    }
}
```

> Add `use crate::application::auth::refresh::RefreshOutput;` and `use crate::application::errors::AppError;` if not already in scope.

Change `logout` to read from body in bearer mode; response shape is unchanged (204 / 503), just no cookies in bearer mode:

```rust
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
        // Nothing coherent to revoke → idempotent 204 (+ clear cookies on web).
        return if bearer {
            StatusCode::NO_CONTENT.into_response()
        } else {
            (StatusCode::NO_CONTENT, clearing_cookies(&state.cookie_policy)).into_response()
        };
    };

    let uc = LogoutUseCase {
        sessions: state.sessions.clone(),
        access_ttl_secs: state.access_ttl_secs,
    };

    match uc.execute(family_id, family_id).await {
        Ok(()) => {
            if bearer {
                StatusCode::NO_CONTENT.into_response()
            } else {
                (StatusCode::NO_CONTENT, clearing_cookies(&state.cookie_policy)).into_response()
            }
        }
        Err(LogoutError::Unavailable) => StatusCode::SERVICE_UNAVAILABLE.into_response(),
    }
}
```

- [ ] **Step 5: Run the bearer refresh/logout tests + full suite**

Run: `cd apps/backend && cargo nextest run --test auth_test`
Expected: all bearer + cookie tests pass.

- [ ] **Step 6: Verify gate + commit**

```bash
cd apps/backend && cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings
git add apps/backend/src/adapters/inbound/http/handlers/auth.rs apps/backend/tests/auth_test.rs
git commit -m "feat(auth): bearer-mode refresh + logout (body refresh token, no cookies)"
```

---

### Task 5: Document the contract in OpenAPI

**TDD: no** — documentation; verified by a YAML lint / manual read.

**Files:**
- Modify: `packages/openapi/veyra.yaml`

- [ ] **Step 1: Add the `X-Auth-Mode` header parameter** to `POST /auth/register`, `/auth/login`, `/auth/refresh`, `/auth/logout` (optional, enum `[bearer]`, description: selects bearer-token delivery).

- [ ] **Step 2: Add component schemas** `AuthTokens`, `AuthResponse`, `RefreshResponse`, `BearerRefreshRequest` matching the DTOs in Task 1. Model the two register/login response bodies via `oneOf: [UserResponse, AuthResponse]` (or per-mode examples).

- [ ] **Step 3: Add a `bearerAuth` security scheme** (`type: http, scheme: bearer, bearerFormat: JWT`) and list it alongside the existing cookie scheme on protected routes; document that bearer requests skip CSRF.

- [ ] **Step 4: Validate the YAML**

Run: a YAML/OpenAPI lint if available (e.g. `npx @redocly/cli lint packages/openapi/veyra.yaml`), otherwise parse-check it.
Expected: no schema errors.

- [ ] **Step 5: Commit**

```bash
git add packages/openapi/veyra.yaml
git commit -m "docs(openapi): document dual-mode auth (X-Auth-Mode + bearer schemas)"
```

---

### Task 6: Full regression + quality-gate verification

**TDD: n/a** — verification task; no new behavior.

- [ ] **Step 1: Format + lint**

Run: `cd apps/backend && cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings`
Expected: clean.

- [ ] **Step 2: Full test run (cookie regression + bearer suite)**

Run: `cd apps/backend && cargo nextest run` (Docker must be running for testcontainers)
Expected: all green — every pre-existing cookie-mode test unchanged + all new bearer-mode tests passing.

- [ ] **Step 3: Confirm the additive invariant by inspection**

Confirm no cookie-mode test was modified to accommodate bearer mode (only additions). If any cookie test changed behavior, that is a regression — investigate before proceeding.

- [ ] **Step 4: Final commit (if any fixups)**

```bash
git add -A apps/backend
git commit -m "test(auth): verify dual-mode auth regression + bearer suite green"
```

---

## Self-Review

- **Spec coverage:** mode selection (Task 1) · register/login bearer (Task 2) · require_auth bearer + require_csrf bypass incl. BUG-#1 X-Auth-Mode discriminator (Task 3) · refresh/logout bearer (Task 4) · OpenAPI (Task 5) · regression (Task 6). All spec sections mapped.
- **Placeholder scan:** none — all steps carry real code or exact commands.
- **Type consistency:** `AuthTokens`/`AuthResponse`/`RefreshResponse`/`BearerRefreshRequest` defined in Task 1, consumed in 2/4; `parse_refresh_value` defined in Task 4 Step 1, reused by `read_refresh` + `refresh_from_body`; `wants_bearer` defined Task 1, used in handlers + csrf; `AuthSession`/`RefreshOutput`/`AuthOutcome` field names match the source files read during planning.
- **Error shape:** bearer 401 = `AppError::Unauthorized.into_response()` → flat `{"error":"unauthorized"}` (matches actual `errors.rs`).
