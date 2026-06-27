# VEYRA

```
 __   __  _______  __   __  ______    _______ 
|  | |  ||       ||  | |  ||    _ |  |   _   |
|  |_|  ||    ___||  |_|  ||   | ||  |  |_|  |
|       ||   |___ |       ||   |_||_ |       |
|       ||    ___||_     _||    __  ||       |
 |     | |   |___   |   |  |   |  | ||   _   |
  |___|  |_______|  |___|  |___|  |_||__| |__|
```

**Open-source vehicle management API built with Rust.**

![Rust](https://img.shields.io/badge/rust-1.82+-orange?style=flat-square)
![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)
![CI](https://img.shields.io/github/actions/workflow/status/oksasatya/veyra/ci.yml?style=flat-square)
![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen?style=flat-square)

Track your vehicles, services, fuel, and expenses — all in one clean API.

---

## Architecture

Hexagonal DDD (Ports & Adapters) in a single Rust crate. The domain layer is
framework-free and tested in isolation. A CI script enforces that `domain/`,
`application/`, and `ports/` never import `axum`, `sqlx`, or `serde`.

### Layer diagram

```mermaid
flowchart LR
  FE["Flutter app · iOS + Android"]:::ext

  subgraph RW["Backend · Railway — api.veyra.dev"]
    direction TB

    subgraph IN["Inbound (driving) adapter"]
      H["adapters/inbound/http<br/>handlers · auth + CSRF middleware<br/>cookies · DTOs"]:::adapter
    end

    A["application<br/>use cases"]:::app
    P["ports<br/>repository · auth · session traits"]:::port
    D["domain<br/>entities · value objects<br/>pure — no I/O"]:::core

    subgraph OUT["Outbound (driven) adapters"]
      PGA["postgres<br/>sqlx repositories"]:::adapter
      RSA["redis<br/>session store · cache decorators"]:::adapter
      TKA["token<br/>JWT sign / verify"]:::adapter
    end

    BS["bootstrap<br/>Config · AppState · DI wiring"]:::boot
  end

  PG[("PostgreSQL")]:::infra
  RD[("Redis")]:::infra

  FE -->|"Authorization: Bearer · X-Auth-Mode"| H
  H --> A
  A --> P
  A --> D
  P --> D
  PGA -. implements .-> P
  RSA -. implements .-> P
  TKA -. implements .-> P
  PGA --> PG
  RSA --> RD
  BS -. assembles .-> H

  classDef ext fill:#e0e7ff,stroke:#6366f1,color:#1e1b4b;
  classDef adapter fill:#fef3c7,stroke:#d97706,color:#78350f;
  classDef app fill:#dcfce7,stroke:#16a34a,color:#14532d;
  classDef port fill:#f1f5f9,stroke:#475569,color:#0f172a;
  classDef core fill:#fee2e2,stroke:#dc2626,color:#7f1d1d;
  classDef infra fill:#e2e8f0,stroke:#475569,color:#0f172a;
  classDef boot fill:#ede9fe,stroke:#7c3aed,color:#4c1d95;

  style RW fill:#ffffff,stroke:#cbd5e1,color:#334155
  style IN fill:#f8fafc,stroke:#e2e8f0,color:#475569
  style OUT fill:#f8fafc,stroke:#e2e8f0,color:#475569
```

Dependency rule (CI-enforced): arrows point **inward** — inbound HTTP → application → ports → domain; outbound adapters *implement* ports. The `domain` core imports nothing outward; `bootstrap` is the only layer that wires concretes together.

### Hexagonal layer boundaries (CI-enforced)

| Layer | May import | Forbidden |
|---|---|---|
| `domain/` | stdlib, thiserror, uuid, chrono, rust_decimal | axum, sqlx, serde, tokio |
| `application/` | domain, ports | axum, sqlx |
| `ports/` | domain only | axum, sqlx, serde |
| `adapters/inbound/http/` | application, ports, axum, serde | sqlx directly |
| `adapters/outbound/postgres/` | ports, sqlx | axum |
| `adapters/outbound/redis/` | ports, fred, sha2 | axum |
| `bootstrap/` | all | — |

---

## Features

- Multi-vehicle tracking per account
- Service history with cost tracking
- Fuel consumption logs with efficiency metrics
- Expense categorization (tire, battery, tax, insurance, other)
- Maintenance reminders (by date, odometer, or both)
- Document tracker (STNK, BPKB, insurance — expiry alerts)
- Per-vehicle dashboard summary (cached in Redis)
- Secure auth with rotating refresh tokens: bearer tokens for the Flutter mobile client (cookie + CSRF flow also supported for browsers)

---

## Quick Start

```bash
git clone https://github.com/oksasatya/veyra && cd veyra
cp apps/backend/.env.example apps/backend/.env
# Edit .env: set DATABASE_URL, REDIS_URL, JWT_SECRET (min 32 chars)
docker compose up -d
# Wait for the health check to pass:
curl http://localhost:8080/health
# {"status":"ok","version":"0.1.0"}
```

---

## Tech Stack

| Layer | Tech |
|---|---|
| Runtime | tokio |
| Web | axum 0.8 + axum-extra 0.10 (cookie) |
| Database | PostgreSQL 17 + sqlx 0.8 |
| Cache / Session | Redis + fred 10 |
| Auth | JWT (jsonwebtoken 9) + Argon2id + rotating refresh tokens (cookie + bearer) |
| Config | figment |
| Testing | cargo nextest + testcontainers (Postgres + Redis) |
| Mobile client | Flutter (Dart) — iOS + Android, in `apps/mobile/` |

---

## API Overview

> The **Auth** column shows the cookie-flow requirement (shipped). The Flutter mobile client
> authenticates with `Authorization: Bearer` + `X-Auth-Mode: bearer` and does **not** send CSRF —
> see [Authentication](#authentication).

| Method | Path | Auth | Description |
|---|---|---|---|
| POST | /auth/register | — | Register; sets access, refresh, and CSRF cookies |
| POST | /auth/login | — | Login; sets access, refresh, and CSRF cookies |
| POST | /auth/refresh | refresh cookie + CSRF | Rotate refresh token; issues new access token |
| POST | /auth/logout | access cookie + CSRF | Revoke session; clears all cookies |
| GET | /me | access cookie | Current user info |
| GET / POST | /vehicles | access cookie + CSRF | List / create vehicles |
| GET / PUT / DELETE | /vehicles/{id} | access cookie + CSRF | Get / update / delete |
| GET | /vehicles/{id}/summary | access cookie | Dashboard aggregation (cached) |
| GET / POST | /vehicles/{id}/services | access cookie + CSRF | Service history |
| GET / POST | /vehicles/{id}/fuel-logs | access cookie + CSRF | Fuel logs |
| GET / POST | /vehicles/{id}/expenses | access cookie + CSRF | Expenses |
| GET / POST | /vehicles/{id}/reminders | access cookie + CSRF | Reminders |
| PATCH | /vehicles/{id}/reminders/{rid} | access cookie + CSRF | Mark reminder complete |
| GET / POST | /vehicles/{id}/documents | access cookie + CSRF | Documents |
| GET | /health | — | Liveness probe |

---

## Authentication

Veyra runs **short-lived access tokens + rotating refresh tokens** over one shared session core, with
**two delivery modes**:

- **Cookie + CSRF** (browsers) — tokens as HttpOnly cookies; the double-submit CSRF flow below. *Shipped.*
- **Bearer** (native mobile) — opt-in via the `X-Auth-Mode: bearer` request header; tokens are returned
  in the JSON body and replayed as `Authorization: Bearer <access>`. CSRF is skipped (no cookie surface).
  Specified in [ADR-0007](docs/adr/0007-dual-mode-auth-bearer-mobile.md); in progress for the Flutter client.

The token model and rotation below are identical across both modes — only delivery and extraction differ.

### Token model

| Token | Form | Lifetime | Transport |
|---|---|---|---|
| Access | JWT HS256, claims `{ sub, sid, jti, iat, exp }` | 15 min (configurable) | HttpOnly cookie |
| Refresh | Opaque `{family_id}.{raw_secret}` | 7 days (configurable) | HttpOnly cookie, `Path=/auth` |
| CSRF | Random base64url, readable by JS | >= refresh lifetime | Non-HttpOnly cookie |

The access JWT embeds `sid` — the refresh family ID. A single `revoked:{sid}` Redis key invalidates
all access tokens of a session simultaneously (reuse detected, logout, or explicit revoke).

### CSRF protection

All mutating protected routes require an `X-CSRF-Token` header that matches the `veyra_csrf` cookie
value (double-submit pattern). The `/auth/register` and `/auth/login` endpoints are exempt (no session
exists yet). The `/auth/refresh` and `/auth/logout` endpoints enforce CSRF.

### Refresh rotation

`POST /auth/refresh` atomically rotates the refresh secret (Lua CAS on Redis). Each rotation
promotes the previous secret to a short grace window (default 10 s) so that a concurrent legitimate
request or a lost-response network retry does not trigger false theft detection. A token outside the
grace window matching neither the current nor previous secret is classified as reuse — the family is
revoked and all access tokens of that session become invalid.

### Cookie prefix matrix

Cookie name prefix is derived from the environment — not configured directly:

| Config | `COOKIE_SECURE` | `COOKIE_SAMESITE` | `COOKIE_DOMAIN` | Resulting prefix |
|---|---|---|---|---|
| Local HTTP dev | `false` | `strict` | unset | none (no `Secure` over HTTP) |
| Self-host HTTPS | `true` | `strict` | unset | `__Host-` |
| Prod subdomain split | `true` | `lax` | `veyra.dev` | `__Secure-` |

Cookie names: `[prefix]veyra_access`, `[prefix]veyra_refresh` (always `Path=/auth`),
`[prefix]veyra_csrf`.

---

## Configuration

All configuration is read from environment variables. Defaults are shown where applicable.

| Variable | Default | Purpose |
|---|---|---|
| `DATABASE_URL` | required | PostgreSQL connection string |
| `REDIS_URL` | required | Redis connection string (`redis://…`) |
| `JWT_SECRET` | required, min 32 bytes | HMAC-SHA256 signing key for access JWTs |
| `PORT` | `8080` | Port the HTTP server binds to |
| `ACCESS_TTL_SECS` | `900` | Access token lifetime in seconds (15 min) |
| `REFRESH_TTL_SECS` | `604800` | Refresh token lifetime in seconds (7 days) |
| `REFRESH_GRACE_SECS` | `10` | Grace window for in-flight refresh retries |
| `COOKIE_SECURE` | `true` | Set the `Secure` flag on cookies; set `false` for local plain-HTTP dev |
| `COOKIE_SAMESITE` | `strict` | Cookie SameSite policy: `strict`, `lax`, or `none` |
| `COOKIE_DOMAIN` | unset | Cookie `Domain` attribute; set to `veyra.dev` for the prod subdomain split |
| `CORS_ALLOWED_ORIGINS` | optional | Comma-separated allowed origins for browser clients; wildcard `"*"` is rejected. Not used by the native mobile client (CORS does not apply). |

---

## Local Development

Docker is required for both the database and the integration test suite.

```bash
# Start Postgres and Redis:
docker compose up -d

# Run the server (from apps/backend/):
cargo run

# Run tests (requires Docker for testcontainers):
cargo nextest run

# Lint and format:
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

The `docker-compose.yml` at the repo root starts:

- PostgreSQL 17 on port 5432 (published for local tooling)
- Redis with `appendonly yes` (no published port — accessed only by the backend and testcontainers)

---

## Deployment

### Railway (backend)

The `railway.toml` at the repo root configures the backend service:

```toml
[build]
builder = "dockerfile"
dockerfilePath = "apps/backend/Dockerfile"

[deploy]
healthcheckPath = "/health"
restartPolicyType = "on_failure"
```

Steps:

1. Create a Railway project and add a new service. Set the service **Root Directory** to the repo root.
2. Attach a managed **PostgreSQL** database — Railway injects `DATABASE_URL` automatically.
3. Attach a managed **Redis** database — Railway injects `REDIS_URL` automatically.
4. Set the following environment variables on the service:

| Variable | Production value |
|---|---|
| `JWT_SECRET` | A strong random string, at least 32 characters |
| `COOKIE_DOMAIN` | `veyra.dev` |
| `COOKIE_SAMESITE` | `lax` |
| `COOKIE_SECURE` | `true` |
| `CORS_ALLOWED_ORIGINS` | `https://veyra.dev` |

### Mobile client (Flutter)

The Veyra client is a Flutter app (iOS + Android) in `apps/mobile/`, built against the API at
`api.veyra.dev` and distributed via TestFlight / Play Store (or a self-host build). It authenticates
with `Authorization: Bearer` tokens (`X-Auth-Mode: bearer`), storing them in the platform secure store
(Keychain / Keystore); native clients are not subject to CORS. Design: see
`docs/superpowers/specs/2026-06-27-veyra-mobile-app-design.md`.

---

## Roadmap

- [x] v0.1 — Scaffolding + health
- [x] v0.2 — Auth (register, login)
- [x] v0.3 — Vehicle CRUD
- [x] v0.4 — Service records
- [x] v0.5 — Fuel + expense logs
- [x] v0.6 — Reminders
- [x] v0.7 — Dashboard summary
- [x] v0.8 — Redis auth (access + refresh cookies, CSRF, session revocation, read cache)
- [ ] v0.9 — Bearer-mode auth for native clients (ADR-0007) + Flutter mobile app (iOS + Android)
- [ ] v1.0 — OpenAPI 3.1 spec + stable MVP

---

## Contributing

PRs welcome. Open an issue first for significant changes.

## License

MIT — see [LICENSE](LICENSE).
