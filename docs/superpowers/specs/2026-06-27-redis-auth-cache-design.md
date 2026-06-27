# Veyra — Redis Auth (Access + Refresh + Cookies) & Caching Design

**Date:** 2026-06-27
**Status:** Approved (pending implementation plan)
**Supersedes:** ADR-0003 (JWT, No Refresh Token in MVP) — see ADR-0006
**Related:** ADR-0001 (Hexagonal DDD), ADR-0002 (Single-User Per Install)

> Cross-model reviewed: design drafted by Claude, adversarially reviewed by Codex (GPT-5.5).
> Sixteen objections raised; the converged design below incorporates all accepted refinements
> (separate refresh secret + hashing, atomic rotation with grace window, session-id revocation,
> fail-closed writes, per-user cache keys with a version counter, cache kept out of `ports/`,
> rotation policy split between port and application, `__Host-`/`__Secure-` cookie prefixes).

---

## 1. Goal

Replace the single long-lived JWT (ADR-0003) with a production-grade authentication scheme —
short-lived access token + rotating refresh token, delivered as HttpOnly cookies, backed by Redis
for session storage, token revocation, and read-through caching of slow-changing data.

This is a portfolio project; demonstrating a correct, production-grade Redis-backed auth + cache
layer is an explicit goal. The "it is overkill for a single user" trade-off (ADR-0003 rationale) is
acknowledged and consciously accepted.

---

## 2. Context & Supersession

ADR-0003 chose a long-lived 7-day JWT with no refresh token, listing the access+refresh+rotation
design as a "Future action" should Veyra need revocation. This document triggers that future action.

Two divergences from ADR-0003's anticipated future action are deliberate:

1. **Redis instead of a `refresh_tokens` Postgres table.** The refresh session store, the access-token
   revocation set, and the read cache all want the same primitive: keyed values with TTL. Redis is the
   natural fit, avoids new migrations, and demonstrates a real Redis integration for the portfolio.
2. **Session-id (`sid`) revocation instead of per-token denylist.** See §4.4.

ADR-0003 is marked **Superseded by ADR-0006**. The original spec's auth section and the
"Refresh token rotation" non-goal are now in scope and superseded by this document.

---

## 3. Token Model

| Token | Form | Lifetime | Stored | Transport |
|---|---|---|---|---|
| **Access** | JWT HS256, claims `{ sub, sid, jti, iat, exp }` | `ACCESS_TTL_SECS` (default 900 = 15 min) | stateless (not stored) | cookie, HttpOnly |
| **Refresh** | opaque `{family_id}.{raw_secret}` (`raw_secret` = 32 random bytes, base64url) | `REFRESH_TTL_SECS` (default 604800 = 7 days) | Redis `session:{family_id}` (stores `hash(raw_secret)`, never the raw value) | cookie, HttpOnly, scoped `Path=/auth` |
| **CSRF** | 32 random bytes, base64url | ≥ refresh lifetime | not stored (double-submit) | cookie, **readable by JS** (not HttpOnly) |

### 3.1 Cookie attributes (environment-configurable)

The same binary must serve both self-host (single origin) and the production split
(`veyra.dev` + `api.veyra.dev`, same registrable domain). Cookie policy is therefore env-driven:

| Env | Self-host default | Prod (subdomain) |
|---|---|---|
| `COOKIE_SECURE` | `true` | `true` |
| `COOKIE_SAMESITE` | `strict` | `lax` |
| `COOKIE_DOMAIN` | unset | `veyra.dev` |

Cookie name prefix is **derived**, not configured:
- `COOKIE_DOMAIN` unset + `COOKIE_SECURE=true` → `__Host-` prefix (requires Secure, no Domain, `Path=/`).
- `COOKIE_DOMAIN` set → `__Secure-` prefix (`__Host-` forbids `Domain`).
- `COOKIE_SECURE=false` (local plain-HTTP dev only) → no prefix (browsers drop `Secure` cookies over HTTP).

Resulting cookie names: `…veyra_access`, `…veyra_refresh` (always `Path=/auth`), `…veyra_csrf`.
The refresh cookie's `Path=/auth` means it cannot use `__Host-`; it always uses `__Secure-` (or none in dev).

CSRF protection: **SameSite plus double-submit token**, both layers. SameSite alone does not cover the
cross-site cases the env matrix allows, so the double-submit check is always enforced on mutating routes.

---

## 4. Authentication Flows

### 4.1 Register / Login

1. Validate input; verify credentials (Argon2id) for login; create user for register.
2. Create a new **session family**: generate `family_id` (UUID) + `raw_secret` (32 bytes).
   `SET session:{family_id} = { user_id, current_secret_hash, prev_secret_hash: null, prev_until: null }`
   with TTL = `REFRESH_TTL_SECS`.
3. Mint an access JWT carrying `sid = family_id`.
4. Set cookies: access, refresh (`{family_id}.{raw_secret}`), csrf.
5. Response body: user info (id, email, name). **No token in the body.**

### 4.2 Authenticated request (middleware `require_auth`)

1. Read access token from the access cookie (no `Authorization` header).
2. Verify JWT cryptographically (signature + `exp`). No Redis needed.
3. Revocation check: `EXISTS revoked:{sid}`.
   - Redis **up**, key present → 401.
   - Redis **up**, key absent → allow.
   - Redis **down** → **fail-open**: skip the check, allow, log a warning. Exposure is bounded by the
     short access TTL (a revoked-but-unexpired token works only during the outage window).
4. Inject `user_id` into request extensions.

### 4.3 CSRF middleware (mutating protected routes: POST/PUT/PATCH/DELETE)

Compare the `X-CSRF-Token` request header to the csrf cookie value. Mismatch or missing → 403.
`/auth/register`, `/auth/login`, `/auth/refresh` are exempt (no session yet / protected by SameSite +
the rotation/reuse machinery).

### 4.4 Refresh (`POST /auth/refresh`) — atomic rotation with grace window

Parse `{family_id}.{raw_secret}` from the refresh cookie. The store performs an **atomic** (Lua CAS)
rotation and returns a typed outcome; the **application** decides what each outcome means (so the
security policy is testable without Redis):

```
SessionStore::rotate(family_id, hash(raw_secret)) -> RotateOutcome
  Rotated { user_id, new_raw_secret }   // hash == current_secret_hash, OR
                                        // (hash == prev_secret_hash AND now < prev_until) → rotate
  Reused                                // hash matches neither current nor an in-grace prev → THEFT
  NotFound                              // family absent/expired
```

The atomic script accepts **either** the current secret **or** the previous secret while it is still
within its grace window (`now < prev_until`), and in both cases performs a fresh rotation: generate a new
`raw_secret`, set `prev = the secret that was current` with `prev_until = now + REFRESH_GRACE_SECS`
(default 10s), set `current = new`, and return the new raw secret. Only the **hash** is ever stored — the
new raw secret exists just long enough to be returned and written into the response cookie.

Application `RefreshUseCase`:
- `Rotated { user_id, new_raw_secret }` → mint a new access JWT (same `sid`); set new access cookie,
  refresh cookie (`{family_id}.{new_raw_secret}`), and csrf cookie.
- `Reused` → **reuse detected**: `DEL session:{family_id}` + `SET revoked:{family_id}` (TTL=`ACCESS_TTL_SECS`)
  → 401, clear cookies. Forces full re-login.
- `NotFound` → 401, clear cookies.

Accepting the in-grace previous secret as a valid rotation input (Codex #2, #3) is what prevents two
concurrent legitimate refreshes — or a network retry of a refresh whose response was lost — from being
misclassified as theft and wrongly revoking the family: each request still receives a working new secret,
and the tabs harmlessly leapfrog. A secret older than the in-grace previous (e.g. a replayed stolen token
presented after the grace window) matches neither slot → `Reused` → family revoked. This avoids ever
storing or returning a raw secret from a prior rotation (the store only holds hashes).

### 4.5 Logout (`POST /auth/logout`) — fail-closed

1. `DEL session:{family_id}` (kills refresh) **and** `SET revoked:{sid}` TTL=`ACCESS_TTL_SECS`
   (kills every access JWT of this session, not just the current `jti`).
2. **If either Redis write fails → 503 "logout degraded, retry".** Do not pretend success by only
   clearing the local cookie (Codex #5): a silently-failed logout leaves the session live when Redis
   returns. Logout is a write and must fail-closed.
3. On success, clear all three cookies (with byte-identical attributes to how they were set).

### 4.6 Why `sid` revocation, not per-`jti` denylist

A single logout or family-revoke must invalidate **all** access tokens minted for that session within
the last `ACCESS_TTL_SECS` (multiple refreshes can issue several). Embedding `sid` in the access JWT and
checking one `revoked:{sid}` key (Codex #4, #6) kills them all in a single lookup — strictly simpler and
more correct than tracking individual `jti`s. `jti` remains in the JWT for logging/traceability only.

---

## 5. Hexagonal Mapping

**Ports (`ports/`, import domain only — no `serde`, no `sqlx`, no `axum`):**
- `auth.rs` — extend `AuthPort`: `sign_access(user_id, sid, jti) -> String`,
  `verify_access(&str) -> Result<AccessClaims, AuthError>` where `AccessClaims { user_id, sid, jti }`.
- `session.rs` (new) — `SessionStore`:
  `create(user_id) -> NewSession{ family_id, raw_secret }`,
  `rotate(family_id, secret_hash) -> RotateOutcome` (`Rotated{ user_id, new_raw_secret } | Reused | NotFound`),
  `revoke(family_id)`, `is_session_revoked(sid) -> bool`.
  `RotateOutcome` is a plain enum over domain/std types (no serde). The grace-window logic and SHA-256
  hashing live inside the Redis adapter's Lua script; the application maps outcomes to actions.

**Adapters (`adapters/outbound/`):**
- `token/jwt_auth.rs` — `JwtAuth` (moved out of `postgres/`; it is not a Postgres concern).
  Implements the extended `AuthPort`.
- `redis/client.rs` — `fred` connection pool, constructed from `REDIS_URL`.
- `redis/session_store.rs` — `RedisSessionStore` implements `SessionStore`. Owns the Lua CAS script for
  atomic rotate, the hashing of `raw_secret` (SHA-256), and `revoked:{sid}` writes.
- `redis/cache.rs` — `RedisCache`, an **adapter-internal** cache abstraction. **Not a port** (Codex #10):
  caching is transparent to use cases, so no port is needed and `serde` stays out of `ports/`.

**Caching repository decorators (`adapters/outbound/redis/cached_*.rs`):**
- `CachedVehicleRepo { inner: Arc<dyn VehicleRepository>, cache }` — implements `VehicleRepository`
  (Codex #9: `inner` is the trait object, not the concrete Pg repo).
- `CachedSummaryRepo { inner: Arc<dyn SummaryRepository>, cache }` — TTL-only (60s), no invalidation.
- Each decorator owns a private `serde` mirror of the domain entity (adapters may use `serde`); the
  domain entity itself stays `serde`-free, preserving the boundary.

---

## 6. Caching Design

**Strategy:** read-through cache-aside inside the decorator; writes invalidate.

**Keys (per-user — Codex #7, prevents cross-user leak):**
- detail: `cache:v{ver}:vehicle:{user_id}:{vehicle_id}`
- list: `cache:v{ver}:vehicles:{user_id}`
- summary: `cache:summary:{user_id}:{vehicle_id}` (TTL 60s, version-independent)

**Invalidation by version counter (Codex #8):** `cache:ver:{user_id}` is a per-user integer.
Read keys embed the current `{ver}`. Any vehicle write (`insert`/`update`/`delete`) does
`INCR cache:ver:{user_id}` → all of that user's old read keys become unreachable and TTL-expire.
This avoids enumerating or missing individual keys. Summary uses TTL-only (the user accepted ≤60s
staleness); it is not version-prefixed.

**Cache-down behavior:** all cache operations fail-open to the inner repo (a Redis outage degrades to
direct Postgres reads, never an error). Cache is never authoritative.

---

## 7. Configuration Additions

| Var | Default | Notes |
|---|---|---|
| `REDIS_URL` | — (required) | `redis://:pass@host:6379` |
| `ACCESS_TTL_SECS` | `900` | 15 min |
| `REFRESH_TTL_SECS` | `604800` | 7 days |
| `REFRESH_GRACE_SECS` | `10` | concurrent-refresh grace window |
| `COOKIE_SECURE` | `true` | `false` only for local plain-HTTP dev |
| `COOKIE_SAMESITE` | `strict` | `lax` for prod subdomain split |
| `COOKIE_DOMAIN` | unset | `veyra.dev` in prod |

Startup validation: `REDIS_URL` present and parseable; existing `JWT_SECRET ≥ 32 bytes` guard retained.

---

## 8. Crates

- `fred` — async Redis client with built-in pooling (per global Rust stack).
- `axum-extra` (feature `cookie`) — `CookieJar` / typed cookie building.
- `sha2` — hashing refresh secrets before storage.
- `rand` — refresh `raw_secret` and CSRF token generation (CSPRNG).
- dev: `testcontainers-modules` feature `redis` — Redis container in integration tests.

Exact versions verified via Context7 during planning.

---

## 9. Deployment

**Local dev / single-box self-host:** `docker compose up`. Compose runs the app, Postgres, and Redis.
Redis: `redis:7-alpine --appendonly yes --requirepass …`, a named volume for persistence (refresh
sessions survive restart; cache loss on restart is harmless), and **no published port** (internal
compose network only). Cookie defaults: `SameSite=Strict`, `__Host-`, no `COOKIE_DOMAIN`.

**Production:** backend on Railway, frontend (future) on Vercel, under one custom domain:
- `api.veyra.dev` → Railway service. Build the existing `apps/backend/Dockerfile`; configure the service
  Root Directory = `apps/backend` (or a root `railway.toml` with `build.dockerfilePath`). Managed Railway
  Postgres + Redis inject `DATABASE_URL` + `REDIS_URL`. **No root Dockerfile and no compose in prod.**
- `veyra.dev` → Vercel (the future `apps/web` Next.js client). Separate deploy, native Next.js build.
- Same registrable domain (`veyra.dev`) → same-site → cookies work with
  `COOKIE_DOMAIN=veyra.dev`, `COOKIE_SAMESITE=lax`, `__Secure-` prefix, double-submit CSRF on.
- **CORS:** explicit allowlist of the Vercel origin + `Allow-Credentials: true`. Never wildcard
  (forbidden by project rules; also illegal alongside credentials).

The application reads only `DATABASE_URL` / `REDIS_URL` / cookie env, so the identical image runs in
self-host and on Railway; only environment differs.

---

## 10. Testing Strategy

- **Unit (no Redis):** `RefreshUseCase` outcome mapping (Rotated / GraceReplay / Reused / NotFound) via a
  fake `SessionStore`; cookie-attribute derivation from the env matrix.
- **Integration (testcontainers Postgres + Redis):**
  - register/login set cookies; `axum-test` `TestServer::save_cookies()` carries them across requests.
  - refresh rotates (old secret rejected after grace); concurrent refresh — second request presents the
    in-grace previous secret → still `Rotated` (gets a working new secret), family NOT revoked; reused
    (post-grace) secret → family revoked, subsequent refresh 401.
  - logout revokes `sid` → all access tokens of that session 401; logout when Redis unreachable → 503.
  - CSRF: mutating request without/with-wrong `X-CSRF-Token` → 403.
  - cache: list hit after first read; write bumps version → next read misses → fresh data; cross-user
    key isolation; Redis-down read falls through to Postgres.
- **Test helper changes (`tests/common/mod.rs`):** `register_and_login` no longer returns a token string
  (cookies are stored in the client jar); add a helper to read the csrf cookie and build the
  `X-CSRF-Token` header for mutating requests. All existing integration tests migrate from the Bearer
  header to the cookie jar.

---

## 11. Security Summary

- Refresh secret is high-entropy and **hashed at rest** in Redis; the `jti`/`family_id` are identifiers,
  never credentials (Codex #1).
- Rotation is atomic (Lua CAS) with a grace window → no wrongful revocation on concurrency/retry
  (Codex #2, #3); genuine replay → full family revocation (theft response).
- Read path fail-open (availability, bounded by short access TTL); write path (logout/refresh)
  fail-closed (Codex #5).
- `__Host-`/`__Secure-` prefixes block subdomain cookie injection (Codex #12); double-submit CSRF always
  enforced on mutations, independent of SameSite.
- Per-user cache keys prevent cross-user leakage (Codex #7).
- `#![forbid(unsafe_code)]`, no `.unwrap()`/`.expect()`/`panic!` on production paths, clippy `-D warnings`,
  parameterized SQL — all retained from the existing quality gate.

---

## 12. Out of Scope (this iteration)

- "Logout all other sessions" management UI / endpoint (the data model supports it; no endpoint yet).
- Per-device session naming / session list endpoint.
- Sliding-window rate limiting on `/auth/*` (separate concern; Redis makes it cheap later).
- The Vercel frontend itself (`apps/web`) — this document only makes the backend ready for it.
