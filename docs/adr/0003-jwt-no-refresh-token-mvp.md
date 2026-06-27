# ADR-0003: JWT Authentication, No Refresh Token in MVP

**Date:** 2026-06-27
**Status:** Accepted
**Deciders:** Oksa Satya

---

## Context

Veyra needs an authentication mechanism for its REST API. The common patterns are:

1. **Short-lived access token + refresh token pair** — access token (15 min – 1 hr), refresh token (30 days); requires a token refresh endpoint and a refresh token store (database or Redis)
2. **Long-lived JWT, no refresh** — one token with a longer expiry (7–30 days); simpler but tokens cannot be revoked before expiry
3. **Session cookie + server-side session store** — traditional session management; requires a session table or Redis

---

## Decision

Use **long-lived JWT (7 days), HS256, no refresh token** for MVP.

- Algorithm: HS256 (jsonwebtoken crate)
- Payload: `{ "sub": "<user_uuid>", "exp": <unix_timestamp> }`
- Expiry: 7 days
- Password hashing: Argon2id (argon2 crate)
- No refresh endpoint, no refresh token table

---

## Rationale

Veyra is a **single-user, self-hosted personal tool**. The risk profile of a refresh token system (token theft, revocation, store management) is disproportionate to the threat model of a personal API running on localhost or a private server.

A 7-day JWT is acceptable because:
- The user is the only person using the system
- Re-login every 7 days is not a significant UX burden for a personal tool
- No refresh token database table or Redis dependency is needed in MVP
- The implementation is simpler, easier to understand in a portfolio context

---

## Consequences

### Positive

- No refresh token table, no Redis, no `/auth/refresh` endpoint
- Simpler middleware (stateless JWT verification only)
- Easier to document and understand

### Negative / Trade-offs

- Tokens cannot be revoked before expiry (e.g., after a password change)
- If the JWT secret is rotated, all active tokens are immediately invalidated (acceptable for personal use)
- Not suitable for a multi-user or SaaS context without adding revocation

### Future action

If Veyra evolves to multi-user or SaaS, implement:
1. Short-lived access token (15 minutes)
2. Refresh token stored in `refresh_tokens` table (hashed, with `user_id`, `expires_at`, `revoked_at`)
3. Token rotation on every refresh
4. Revocation on password change or explicit logout
