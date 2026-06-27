# ADR-0007: Dual-Mode Authentication — Bearer Tokens for Native Mobile

**Date:** 2026-06-27
**Status:** Accepted
**Extends:** ADR-0006 (Refresh Tokens, Redis Session Store, and Read Caching)
**Deciders:** Oksa Satya

> **Extends, does not supersede, ADR-0006.** The cookie + CSRF flow stays exactly as ADR-0006
> designed it and remains the default for browser clients. This ADR adds a second, opt-in delivery
> path — `Authorization: Bearer` tokens in the JSON body — for native mobile clients (the planned
> Flutter app). The underlying session model (access JWT + rotating Redis refresh family + sid
> revocation) is unchanged and shared by both paths.

---

## Context

ADR-0006 moved Veyra to a browser-first authentication model: access and refresh tokens delivered as
`HttpOnly` cookies, double-submit CSRF on mutating routes, `__Host-`/`__Secure-` cookie-name prefixes,
and `require_auth` reading the **access cookie** rather than the `Authorization` header. That model is
correct for a web client — `HttpOnly` removes the token from JavaScript reach (XSS mitigation) and CSRF
+ SameSite defends the cookie.

Veyra is now growing a **native mobile client** (Flutter, iOS + Android — see the mobile design spec).
The browser-first model is a poor fit for a native client:

1. **Cookie ergonomics.** A native HTTP client must run a manual cookie jar; refresh-cookie `Path=/auth`
   scoping and `__Host-` rules are browser concepts that add friction with no security benefit on
   native.
2. **`__Host-`/`__Secure-` require HTTPS.** Local development against `http://localhost:3000` cannot
   set those cookies, so a cookie-only contract breaks the mobile dev loop.
3. **CSRF is a browser threat.** A native app has no ambient-credential / cross-site-request surface, so
   double-submit CSRF is pure overhead for it.
4. **Native secure storage is the idiomatic token home.** iOS Keychain / Android Keystore
   (`flutter_secure_storage`) are designed exactly for bearer-token-at-rest; there is no DOM/XSS surface
   to protect against the way `HttpOnly` protects a browser.

The alternatives considered:

1. **Force the native client onto the cookie + CSRF flow** — make Flutter run a cookie jar, read the
   non-`HttpOnly` CSRF cookie, and echo it as `X-CSRF-Token`. Zero backend change, but maximal client
   friction and a broken `http://localhost` dev loop.
2. **Dual-mode: add an opt-in Bearer delivery path** *(chosen)* — keep cookies for web, add a header-gated
   Bearer path for native, reusing the same session machinery.
3. **A separate mobile auth service / different token system** — over-engineered for a single-binary
   portfolio project; duplicates the session model.

---

## Decision

Add an **opt-in Bearer delivery mode**, selected by a request header `X-Auth-Mode: bearer`, layered on
top of the existing cookie flow. The session model (access JWT `{ sub, sid, jti, iat, exp }`, Redis
refresh family `{family_id}.{raw_secret}`, sid revocation, Lua-CAS rotation with grace window) is reused
verbatim — only **token delivery and extraction** differ.

### Request/response contract

- **`POST /auth/register`, `POST /auth/login`**
  - Default (no header): unchanged — set `access`/`refresh`/`csrf` cookies, body = `UserResponse`.
  - `X-Auth-Mode: bearer`: do **not** set cookies; body = `{ user, tokens: { access_token, refresh_token } }`
    where `refresh_token` is the opaque `{family_id}.{raw_secret}` string.
- **`POST /auth/refresh`**
  - Default: read refresh from the cookie, rotate, re-issue cookies (unchanged).
  - `X-Auth-Mode: bearer`: read refresh from the JSON body `{ "refresh_token": "{family_id}.{raw_secret}" }`,
    rotate, return `{ tokens: { access_token, refresh_token } }`; set no cookies.
- **`POST /auth/logout`**
  - Default: read refresh from the cookie, revoke, clear cookies (unchanged).
  - `X-Auth-Mode: bearer`: read refresh from the JSON body, revoke, return 204; no cookies.

### Middleware

- **`require_auth`** authenticates by `Authorization: Bearer <access-jwt>` **first**; if present it runs
  the existing `verify_access` → `is_session_revoked(sid)` → inject `user_id` path (identical logic, just
  a different token source). If absent, it falls back to the access-cookie path (unchanged). Fail-open on
  the revocation read is preserved for both.
- **`require_csrf`** is **bypassed when the request carries an `Authorization` header OR `X-Auth-Mode:
  bearer`** (bearer/native = no cookie = no CSRF surface). The `X-Auth-Mode` branch is required because
  bearer-mode `/auth/refresh` and `/auth/logout` run after the access token has expired — they send
  `X-Auth-Mode: bearer` + a body refresh token and no `Authorization` header, so an `Authorization`-only
  bypass would 403 them. Cookie-authenticated requests keep the existing double-submit enforcement.

### Token-at-rest on the client

The native client stores `access_token` + `refresh_token` in `flutter_secure_storage` (Keychain /
Keystore). The backend imposes no storage requirement beyond "treat the refresh token as a secret".

---

## Rationale

- **Reuses, does not fork, the session model.** Rotation, reuse-detection, sid-revocation, and
  fail-mode policy are identical across web and mobile; only delivery/extraction is mode-specific. One
  source of truth for session security.
- **No regression to the web posture.** Bearer tokens are only emitted when a client explicitly opts in
  via `X-Auth-Mode: bearer`; a browser never receives a token in the body, so `HttpOnly` + CSRF stay
  intact for web. The change is strictly additive.
- **Idiomatic for each platform.** Web gets cookie/CSRF; native gets Bearer + secure storage — each the
  documented best practice for its surface.
- **Portfolio value.** Demonstrating a single session core serving two delivery models (cookie web +
  bearer mobile) without duplicating security logic is itself a showcase.

---

## Consequences

### Positive

- Native mobile dev loop works against `http://localhost` (no `__Host-`/HTTPS cookie requirement on the
  bearer path).
- Web security posture (ADR-0006) is unchanged.
- Session security logic stays single-sourced; web and mobile cannot diverge.
- CSRF overhead is correctly skipped where it has no threat to defend.

### Negative / Trade-offs

- A second, header-gated code path in the auth handlers and two middlewares — more branches to test
  (mitigated by integration tests covering both modes).
- The bearer refresh token lives in client storage rather than an `HttpOnly` cookie; acceptable because
  native secure storage has no XSS-equivalent exposure, and the refresh family is still rotated +
  revocable server-side.
- The contract now has a mode switch (`X-Auth-Mode`); it must be documented in the OpenAPI spec so the
  two response shapes are explicit.

### Migration / Impact

- New DTOs: `AuthTokens { access_token, refresh_token }`; register/login responses become
  `{ user, tokens? }` with `tokens` present only in bearer mode.
- `require_auth`: add an `Authorization: Bearer` branch ahead of the cookie branch.
- `require_csrf`: add an `Authorization`-header bypass.
- `/auth/refresh` + `/auth/logout`: add body-based refresh extraction for bearer mode.
- Refresh-token extraction helper generalized to read from cookie **or** body.
- OpenAPI (`packages/openapi/veyra.yaml`): document `X-Auth-Mode` and both response shapes.
- CORS: irrelevant to native clients (no Origin enforcement on the bearer path); the existing web CORS
  allowlist is untouched.
- Tests: add bearer-mode integration tests alongside the existing cookie-jar tests.
