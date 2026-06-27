# Veyra — Dual-Mode Auth (Bearer for Mobile) Design Specification

**Date:** 2026-06-27
**Status:** Approved
**Author:** Oksa Satya
**ADR:** ADR-0007 (extends ADR-0006)
**Related:** `docs/superpowers/specs/2026-06-27-redis-auth-cache-design.md`

---

## Overview

This spec describes a backend refactor that makes Veyra's authentication serve **two delivery modes**
from one session core:

- **Cookie mode** (existing, web) — `HttpOnly` access/refresh cookies + double-submit CSRF, exactly as
  ADR-0006 defines it. Unchanged.
- **Bearer mode** (new, native mobile) — access + refresh tokens returned in the JSON body and presented
  back as `Authorization: Bearer <access>`. Opt-in via the `X-Auth-Mode: bearer` request header.

The session model is reused verbatim: access JWT `{ sub, sid, jti, iat, exp }`, opaque rotating refresh
`{family_id}.{raw_secret}` backed by Redis, sid revocation, Lua-CAS rotation with a grace window, and
the fail-open-read / fail-closed-write policy. **Only token delivery and extraction change.**

The consumer of bearer mode is the planned Flutter mobile app; that app is designed separately and is
**out of scope** here.

---

## Goals

- Add an opt-in Bearer delivery path for native clients without touching the web cookie/CSRF flow.
- Keep one source of truth for session security (rotation, reuse-detection, revocation, fail modes).
- Make the mobile dev loop work against `http://localhost` (no `__Host-`/HTTPS cookie dependency).
- Document both response shapes in the OpenAPI contract.

## Non-Goals

- The Flutter mobile app itself (separate spec).
- Changing the session model, rotation, revocation, or cache behavior from ADR-0006.
- OAuth / social login / multi-tenant auth.
- Per-`jti` fine-grained revocation (sid revocation stays the model).

---

## Mode Selection

A single request header selects the delivery mode:

```
X-Auth-Mode: bearer
```

- **Absent or any other value** → cookie mode (default; current behavior, byte-for-byte).
- **`bearer`** → bearer mode: tokens travel in the JSON body; no `Set-Cookie` is emitted.

Mode is decided **per request** at the four auth endpoints (`register`, `login`, `refresh`, `logout`).
Protected routes do not need the header — they are disambiguated by the presence of an `Authorization`
header (see Middleware).

---

## Contract Changes

### `POST /auth/register` · `POST /auth/login`

**Cookie mode (unchanged):**
```
Request:  { "email": "...", "password": "...", "name": "..." }   // register
Response: 201/200  Set-Cookie: access, refresh, csrf
          body: { "id": "...", "email": "...", "name": "..." }    // UserResponse
```

**Bearer mode (`X-Auth-Mode: bearer`):**
```
Response: 201/200  (no Set-Cookie)
          body: {
            "user":   { "id": "...", "email": "...", "name": "..." },
            "tokens": { "access_token": "<jwt>", "refresh_token": "<family_id>.<raw_secret>" }
          }
```

### `POST /auth/refresh`

**Cookie mode (unchanged):** refresh cookie → rotate → re-issue cookies (200) / 401+clear / 503.

**Bearer mode:**
```
Request:  X-Auth-Mode: bearer
          { "refresh_token": "<family_id>.<raw_secret>" }
Response: 200  body: { "tokens": { "access_token": "...", "refresh_token": "..." } }   (no cookies)
          401  body: { "error": "unauthorized" }                                        (invalid/reuse)
          503                                                                            (store down)
```

### `POST /auth/logout`

**Cookie mode (unchanged):** refresh cookie → revoke → 204+clear / 503.

**Bearer mode:**
```
Request:  X-Auth-Mode: bearer
          { "refresh_token": "<family_id>.<raw_secret>" }
Response: 204   (no cookies)   — idempotent: malformed/absent body still 204
          503                  — store down (fail-closed)
```

### Protected routes (`GET /me`, `/vehicles/*`, …)

```
Request:  Authorization: Bearer <access-jwt>      (no cookie, no CSRF header needed)
```
Behavior identical to the cookie path: `verify_access` → `is_session_revoked(sid)` → `user_id` injected.

---

## DTO Changes

`apps/backend/src/adapters/inbound/http/dto/auth.rs`:

```rust
/// Access + refresh pair returned only in bearer mode.
#[derive(Debug, Serialize)]
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String, // "{family_id}.{raw_secret}"
}

/// Bearer-mode body for register/login. Cookie mode keeps returning bare UserResponse.
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user: UserResponse,
    pub tokens: AuthTokens,
}

/// Bearer-mode body for refresh.
#[derive(Debug, Serialize)]
pub struct RefreshResponse {
    pub tokens: AuthTokens,
}

/// Body carrying the refresh token for bearer-mode refresh/logout.
#[derive(Debug, Deserialize)]
pub struct BearerRefreshRequest {
    pub refresh_token: String,
}
```

`UserResponse`, `MeResponse`, `RegisterRequest`, `LoginRequest` are unchanged.

---

## Middleware Changes

### `require_auth` (`middleware/auth.rs`)

Add a bearer branch **ahead of** the cookie branch; everything after token extraction is shared:

```text
1. If `Authorization: Bearer <t>` present → token = t            (bearer path)
   else → token = access cookie value, or 401+clear if absent     (cookie path, existing)
2. verify_access(token) → claims, or 401 (+clear only on cookie path)
3. is_session_revoked(claims.sid)  — fail-open on Err (unchanged)
4. inject claims.user_id
```

The bearer path must **not** emit clearing cookies on 401 (there are no cookies); it returns a bare
`401` with the standard error body. The cookie path keeps `unauthorized_clearing`.

### `require_csrf` (`middleware/csrf.rs`)

Add a bypass at the top:

```text
If the request carries an `Authorization` header OR `X-Auth-Mode: bearer` → next
    (bearer/native = no cookie = no CSRF surface).
Otherwise → existing safe-method + double-submit check (unchanged).
```

> **Why both conditions, not just `Authorization`.** Protected routes carry `Authorization: Bearer
> <access>`, so that header alone covers them. But `/auth/refresh` and `/auth/logout` in bearer mode run
> *because the access token has expired* — the client sends `X-Auth-Mode: bearer` + the refresh token in
> the body and **no** `Authorization` header. Without the `X-Auth-Mode` branch, `require_csrf` would fall
> through to the double-submit check (cookie absent, header absent) and reject the request with 403,
> breaking native refresh/logout. Bearer-mode refresh/logout therefore MUST send `X-Auth-Mode: bearer`
> (already mandated by the contract), and the bypass MUST honor it.

---

## Refresh-Token Extraction

Generalize the current `read_refresh(policy, jar)` so refresh/logout can read the token from **either**
source, parsing the same `{family_id}.{raw_secret}` shape (split on first `.`):

- **Cookie mode:** read from the refresh cookie (existing `read_refresh`).
- **Bearer mode:** read from the `BearerRefreshRequest` body.

Parsing/validation (`Uuid` family + non-empty secret) is shared; only the source differs. Keep one
parse helper to satisfy the DRY rule (one refresh-parse function, not two).

---

## File-Level Change Map

```
apps/backend/src/adapters/inbound/http/
├── dto/auth.rs            ← add AuthTokens, AuthResponse, RefreshResponse, BearerRefreshRequest
├── handlers/auth.rs       ← register/login/refresh/logout: branch on X-Auth-Mode;
│                             bearer → body tokens, no cookies; reuse session/use-case calls
├── middleware/auth.rs     ← require_auth: Authorization: Bearer branch before cookie branch
└── middleware/csrf.rs     ← require_csrf: bypass when Authorization header present
packages/openapi/veyra.yaml ← document X-Auth-Mode + both response shapes
apps/backend/tests/auth_test.rs ← add bearer-mode cases beside cookie-jar cases
```

No changes to: application use cases (`register`/`login`/`refresh`/`logout`), ports, domain, Redis
session store, cache, router wiring (the same middlewares apply; their internal branching changes).

> **Note on `register`/`login` mode detection.** The handler reads the `X-Auth-Mode` header (via an
> `axum::http::HeaderMap` extractor or a typed header). To keep the handler under the cognitive-complexity
> budget, extract a small `fn wants_bearer(headers: &HeaderMap) -> bool` helper and a
> `fn bearer_body(user, session) -> AuthResponse` builder rather than inlining branches.

---

## Error Handling

The **actual** error body emitted by `AppError::into_response`
(`adapters/inbound/http/errors.rs`) is:

```json
{ "error": "<message>" }
```

a flat `error` string — **not** the nested `{"error":{"code","message"}}` shape the original main
design spec describes. The implementation is the source of truth; this spec and the Flutter
`Failure`-mapping follow the flat shape. (The main spec's nested shape is stale and out of scope to
reconcile here.)

Status mapping is unchanged (`AppError`: 401 / 404 / 409 / 422 / 500; `require_csrf`: 403; refresh/logout
store-down: 503). Both modes share this mapping. The only delivery difference: bearer-mode 401s reuse
`AppError::Unauthorized.into_response()` (flat body, **no** cookies); cookie-mode 401s use
`unauthorized_clearing` (no body, clears cookies).

---

## OpenAPI Updates (`packages/openapi/veyra.yaml`)

- Add the optional `X-Auth-Mode` header parameter to `register`, `login`, `refresh`, `logout`.
- Model the two response shapes (cookie: `UserResponse`; bearer: `AuthResponse`/`RefreshResponse`) via
  `oneOf` or per-mode examples.
- Document `BearerRefreshRequest` as the bearer-mode body for refresh/logout.
- Document `Authorization: Bearer` as an accepted security scheme on protected routes (alongside the
  cookie scheme).

---

## Testing Strategy

`TDD: yes` — auth has clear input→output contracts and security invariants.

Add bearer-mode integration tests beside the existing cookie tests in `tests/auth_test.rs`:

- register/login with `X-Auth-Mode: bearer` → body has `tokens`, response has **no** `Set-Cookie`.
- protected route with `Authorization: Bearer <access>` → 200; missing/invalid → 401, **no** clearing
  cookies.
- protected route mutation with bearer (no CSRF header) → succeeds (CSRF bypassed).
- refresh (bearer) with a valid refresh body → new `tokens`; reused old refresh → 401 (family revoked).
- logout (bearer) → 204; subsequent refresh with the revoked token → 401.
- **Regression:** the full existing cookie-mode suite stays green unchanged (no header → identical
  behavior).

Verify: `cargo fmt --check → cargo clippy --all-targets --all-features -- -D warnings → cargo nextest run`.

---

## Sequencing

1. DTOs (`AuthTokens`, `AuthResponse`, `RefreshResponse`, `BearerRefreshRequest`).
2. `require_csrf` bypass + `require_auth` bearer branch (+ tests).
3. `register`/`login` bearer branch (+ tests).
4. `refresh`/`logout` bearer extraction + responses (+ tests).
5. OpenAPI contract update.
6. Full regression run (cookie suite + new bearer suite).

This is the prerequisite for the Flutter mobile app; the app's own design (full hexagonal DDD, Riverpod,
screens) is a separate spec to be written next.
