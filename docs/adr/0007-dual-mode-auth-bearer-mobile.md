# ADR-0007: Bearer-Token Authentication for the Native Mobile Client

**Date:** 2026-06-27 (revised 2026-06-28 — narrowed from dual-mode to bearer-only)
**Status:** Accepted
**Extends:** ADR-0006 (Refresh Tokens, Redis Session Store, and Read Caching)
**Deciders:** Oksa Satya

> **Revision note.** This ADR originally introduced a *dual-mode* design — cookie + CSRF for browsers
> alongside an opt-in `Authorization: Bearer` path for native clients. Veyra has no web frontend and
> none is planned: the only client is the Flutter app. The cookie flow, CSRF middleware, CORS layer,
> the `X-Auth-Mode` switch, and the cookie-name-prefix logic were therefore **removed**, leaving a
> single **bearer-only** authentication surface. The session model from ADR-0006 (access JWT +
> rotating Redis refresh family + sid revocation) is unchanged.

---

## Context

ADR-0006 moved Veyra to a browser-first authentication model: access and refresh tokens delivered as
`HttpOnly` cookies, double-submit CSRF on mutating routes, `__Host-`/`__Secure-` cookie-name prefixes,
and `require_auth` reading the **access cookie**.

Veyra's only client is a **native mobile app** (Flutter, iOS + Android — see the mobile design spec).
The browser-first model is a poor fit for a native client, and carrying it alongside a bearer path was
dead weight once the web frontend was dropped from scope:

1. **Cookie ergonomics.** A native HTTP client would have to run a manual cookie jar; refresh-cookie
   `Path=/auth` scoping and `__Host-` rules are browser concepts that add friction with no security
   benefit on native.
2. **`__Host-`/`__Secure-` require HTTPS.** A cookie-only contract breaks the mobile dev loop against
   `http://localhost`.
3. **CSRF is a browser threat.** A native app has no ambient-credential / cross-site-request surface,
   so double-submit CSRF is pure overhead for it.
4. **CORS is a browser threat.** Native HTTP clients are not subject to the same-origin policy, so an
   allowlist protects nothing here.
5. **Native secure storage is the idiomatic token home.** iOS Keychain / Android Keystore
   (`flutter_secure_storage`) are designed exactly for bearer-token-at-rest; there is no DOM/XSS
   surface to protect the way `HttpOnly` protects a browser.

Alternatives considered:

1. **Keep the dual-mode design** (cookie + CSRF for a hypothetical future web client, bearer for
   mobile) — carries a cookie jar, CSRF middleware, CORS layer, and a mode switch that nothing
   exercises. Pure speculative complexity (YAGNI) for a project with one native client.
2. **Bearer-only** *(chosen)* — a single delivery path matched to the only client. Cookie/CSRF/CORS
   code removed; the session machinery is reused verbatim. A future web client, if it ever appears,
   would reintroduce a cookie adapter then — documented in git history.
3. **A separate mobile auth service / different token system** — over-engineered for a single-binary
   portfolio project; duplicates the session model.

---

## Decision

Authenticate **exclusively with bearer tokens**. The session model (access JWT `{ sub, sid, jti, iat,
exp }`, Redis refresh family `{family_id}.{raw_secret}`, sid revocation, Lua-CAS rotation with grace
window) from ADR-0006 is reused verbatim; only token **delivery** changes from cookies to the JSON
body + `Authorization` header.

### Request/response contract

- **`POST /auth/register` (201), `POST /auth/login` (200)** — return
  `{ user, tokens: { access_token, refresh_token } }`, where `refresh_token` is the opaque
  `{family_id}.{raw_secret}` string. No cookies are set.
- **`POST /auth/refresh` (200)** — read the refresh token from the JSON body
  `{ "refresh_token": "{family_id}.{raw_secret}" }`, rotate, return `{ tokens: { access_token,
  refresh_token } }`. A missing / malformed / reused token → 401; session store down → 503.
- **`POST /auth/logout` (204)** — read the refresh token from the JSON body, derive the family
  (`sid == family_id`), revoke it. No coherent token → 204 (idempotent); store down → 503.

### Middleware

- **`require_auth`** authenticates by `Authorization: Bearer <access-jwt>`: `verify_access` →
  `is_session_revoked(sid)` (fail-open on a store read error) → inject `user_id`. Any failure → a bare
  `401`. There is no cookie fallback.
- **No CSRF middleware, no CORS layer.** Both defend browser-only threats that a native client does not
  present. `/auth/refresh` and `/auth/logout` are open routes (no auth layer): they authenticate via
  the refresh token in the body, so they keep working once the access token has expired.

### Token-at-rest on the client

The native client stores `access_token` + `refresh_token` in `flutter_secure_storage` (Keychain /
Keystore). The backend imposes no storage requirement beyond "treat the refresh token as a secret".

---

## Rationale

- **Reuses, does not fork, the session model.** Rotation, reuse-detection, sid-revocation, and
  fail-mode policy from ADR-0006 are untouched — one source of truth for session security.
- **Smallest surface that fits the only client (YAGNI).** No cookie jar, CSRF token, cookie-prefix
  matrix, or CORS allowlist to build, configure, or test, because nothing exercises them.
- **Idiomatic for the platform.** Bearer + secure storage is the documented best practice for native.
- **Portfolio value.** A focused, single-path auth with rotating refresh + Redis revocation reads more
  clearly than a dual-mode flow with a dead branch.

---

## Consequences

### Positive

- The mobile dev loop works against `http://localhost` (no HTTPS-only cookie requirement).
- Less code and fewer dependencies: cookie/CSRF/CORS modules and the `tower-http`, `axum-extra`, and
  `cookie` crates were removed.
- Session security logic stays single-sourced.
- Configuration shrinks to `JWT_SECRET` (+ optional TTL knobs); no `COOKIE_*` / `CORS_*` env.

### Negative / Trade-offs

- A future **web** client cannot reuse this auth as-is; it would need a cookie + CSRF adapter
  reintroduced. Accepted: there is no web client and none is planned, and the prior dual-mode design
  is preserved in git history if it is ever needed.
- The refresh token lives in client storage rather than an `HttpOnly` cookie; acceptable because native
  secure storage has no XSS-equivalent exposure, and the refresh family is still rotated + revocable
  server-side.

### Migration / Impact

- Removed: `cookies.rs`, the CSRF middleware, the CORS layer, the `X-Auth-Mode` header detection, and
  the `COOKIE_*` / `CORS_ALLOWED_ORIGINS` config (plus the `tower-http` / `axum-extra` / `cookie`
  dependencies).
- `require_auth`: bearer-only (cookie branch removed).
- `/auth/refresh` + `/auth/logout`: read the refresh token from the body; they are open routes.
- DTOs: register/login/refresh always return the `tokens` pair.
- Tests: the cookie + CSRF integration tests were converted to bearer.
