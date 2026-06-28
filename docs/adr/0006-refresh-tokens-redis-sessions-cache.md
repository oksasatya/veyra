# ADR-0006: Refresh Tokens, Redis Session Store, and Read Caching

**Date:** 2026-06-27
**Status:** Accepted (token delivery later narrowed by ADR-0007)
**Supersedes:** ADR-0003 (JWT Authentication, No Refresh Token in MVP)
**Deciders:** Oksa Satya

> **Note (2026-06-28):** [ADR-0007](0007-dual-mode-auth-bearer-mobile.md) narrowed token **delivery**
> to **bearer-only** for the native mobile client — the cookie / CSRF / CORS / cookie-prefix details
> described below were removed. The session model in this ADR (access JWT + rotating Redis refresh
> family + sid revocation + read caching) is unchanged and still current; only the transport differs.

---

## Context

ADR-0003 chose a long-lived 7-day JWT with no refresh token and no Redis, explicitly listing an
access+refresh+rotation scheme as a "Future action" if revocation became necessary. We are now
triggering that future action.

The driver is twofold:

1. **Portfolio demonstration.** Veyra is a public portfolio project. A correct, production-grade
   authentication layer — short-lived access tokens, rotating refresh tokens, server-side revocation,
   CSRF defense — and a Redis-backed cache are deliberate showcases of real backend skill. ADR-0003's
   "disproportionate for a single user" rationale is acknowledged and consciously overridden for this
   reason.
2. **Capabilities ADR-0003 could not offer.** Token revocation before expiry (logout, theft response),
   and read caching of slow-changing data.

The full design lives in `docs/superpowers/specs/2026-06-27-redis-auth-cache-design.md` and was
cross-model reviewed (Claude draft, Codex GPT-5.5 adversarial pass, 16 objections folded in).

---

## Decision

Replace the single long-lived JWT with:

- **Access token** — JWT HS256, claims `{ sub, sid, jti, iat, exp }`, ~15 min, stateless, delivered as
  an HttpOnly cookie. `sid` identifies the session family.
- **Refresh token** — opaque `{family_id}.{raw_secret}` (32-byte CSPRNG secret), ~7 days, delivered as an
  HttpOnly cookie scoped `Path=/auth`. The secret is **hashed (SHA-256) before storage**; Redis never
  holds the raw value.
- **Redis session store** — `session:{family_id}` holds `{ user_id, current_secret_hash, prev_secret_hash,
  prev_until }` with TTL = refresh lifetime. Rotation on every refresh is **atomic (Lua CAS)** with a
  short **grace window** so concurrent/retried refreshes are not misread as theft. A genuine replay
  (post-grace mismatch) revokes the entire family (theft response).
- **Session-id revocation** — `revoke:{sid}` (TTL = access lifetime) invalidates every access token of a
  session at once, rather than tracking individual `jti`s.
- **CSRF** — double-submit token (readable cookie + `X-CSRF-Token` header) enforced on all mutating
  routes, in addition to SameSite.
- **Read caching** — transparent caching-repository decorators (`CachedVehicleRepo`, `CachedSummaryRepo`)
  in the adapter layer, with per-user keys and a per-user version counter for invalidation.

### Two deliberate divergences from ADR-0003's anticipated future action

1. **Redis, not a `refresh_tokens` Postgres table.** The refresh session store, the access-token
   revocation set, and the read cache all need keyed-values-with-TTL. Redis serves all three, adds no
   migrations, and is itself a portfolio showcase. A Postgres table would need its own expiry sweeping
   and gives no caching benefit.
2. **Session-id (`sid`) revocation, not a per-token denylist.** One key invalidates all of a session's
   access tokens; simpler and more correct than per-`jti` bookkeeping.

### Fail-mode policy

- **Read path** (access-token revocation check) **fails open** when Redis is down — the app stays up;
  exposure is bounded by the short access TTL.
- **Write path** (logout, refresh) **fails closed** — a logout that cannot reach Redis returns 503 rather
  than silently leaving the session live.

### Cookie policy is environment-configurable

The same binary serves self-host (single origin) and the production subdomain split. `COOKIE_SECURE`,
`COOKIE_SAMESITE`, and `COOKIE_DOMAIN` are env-driven; the `__Host-` / `__Secure-` name prefix is derived.

---

## Consequences

### Positive

- Real logout and theft response (revocation before expiry).
- Short access-token window limits the blast radius of a leaked access token.
- Demonstrates production Redis usage: session store, revocation, caching.
- Read caching for slow-changing vehicle data; cache failures degrade to direct Postgres reads.
- Backend is deploy-portable: identical image on self-host (compose) and Railway (managed Redis/Postgres).

### Negative / Trade-offs

- New hard dependency on Redis for login/refresh/logout (mitigated: read path fails open; managed Redis in prod).
- More moving parts than ADR-0003 (a Lua rotation script, cookie-prefix logic, CSRF middleware, cache
  decorators) — accepted as the cost of the portfolio goal.
- Cross-site cookie constraints require a single registrable domain in production (`veyra.dev` +
  `api.veyra.dev`); a raw `vercel.app` + `railway.app` split would force `SameSite=None`.

### Migration

- New ports: extended `AuthPort`, new `SessionStore`. New adapters: `redis/` (client, session store,
  cache, cached repos); `JwtAuth` moves from `postgres/` to `token/`.
- New endpoints: `POST /auth/refresh`, `POST /auth/logout`. `register`/`login` set cookies instead of
  returning a token in the body. Middleware reads the access cookie instead of the `Authorization` header.
- Integration tests migrate from Bearer header to the `axum-test` cookie jar.
- `docker-compose.yml` gains a Redis service; new env vars per the design doc.
