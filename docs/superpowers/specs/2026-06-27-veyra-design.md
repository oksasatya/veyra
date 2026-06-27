# Veyra — Design Specification

**Date:** 2026-06-27
**Status:** Approved
**Author:** Oksa Satya

---

## Overview

Veyra is an open-source, self-hosted vehicle management REST API built with Rust. It enables individual users to track their vehicles' complete lifecycle: service history, fuel consumption, expenses, maintenance reminders, and documents — all through a clean, well-structured API.

The project is intentionally built as a portfolio-quality public repository that demonstrates real Rust skills: Hexagonal DDD architecture, clean domain modeling, SQLx async database access, and JWT authentication.

---

## Goals

- Showcase real-world Rust backend skills in a public portfolio repository
- Demonstrate Hexagonal DDD architecture with CI-enforced layer separation
- Provide a genuinely useful tool for personal vehicle management
- Serve as a foundation for future Flutter mobile and Tauri desktop clients

## Non-Goals (MVP)

- Multi-tenant or multi-user SaaS operation
- File storage / document upload (URL tracking only in MVP)
- Real-time notifications or push alerts
- OBD-II / telematics hardware integration
- Refresh token rotation (JWT 7-day expiry is sufficient for MVP)

---

## Architecture

### Pattern: Enhanced Single-Crate Hexagonal DDD

The system follows Hexagonal Architecture (Ports & Adapters) within a single Cargo crate. Layer boundaries are enforced by a CI script (`ci/check-boundaries.sh`) that fails the build if forbidden imports appear in constrained layers. A two-crate workspace extraction is documented as a future roadmap item, to be executed once the domain model has stabilized.

**Layer import rules:**

| Layer | May Import | Forbidden |
|-------|-----------|---------|
| `domain/` | stdlib, `thiserror`, `uuid`, `chrono` | `axum`, `sqlx`, `serde`, `tokio` |
| `application/` | `domain`, `ports` | `axum`, `sqlx` |
| `ports/` | `domain` only | `axum`, `sqlx` |
| `adapters/inbound/http/` | `application`, `ports`, `axum`, `serde` | `sqlx` directly |
| `adapters/outbound/postgres/` | `ports`, `sqlx` | `axum` |
| `bootstrap/` | all layers | — |

**Future two-crate extraction (when domain model is stable):**

```
crates/
├── veyra-core/      # domain + application + ports (one lib crate)
└── veyra-adapters/  # inbound/http + outbound/postgres (one lib crate)
apps/backend/        # binary composition root only
```

The move to two crates provides compiler-enforced isolation at the cost of working through Rust's orphan rule for trait implementations. The single-crate approach avoids this overhead during the modeling phase.

---

## Monorepo Layout

```
veyra/
├── apps/
│   ├── backend/
│   │   ├── Cargo.toml
│   │   ├── Cargo.lock
│   │   └── src/
│   │       ├── main.rs
│   │       ├── domain/
│   │       │   ├── vehicle/
│   │       │   │   ├── entity.rs           # Vehicle, VehicleId
│   │       │   │   └── value_objects.rs    # PlateNumber, Odometer, FuelType
│   │       │   ├── service_record/
│   │       │   │   ├── entity.rs
│   │       │   │   └── value_objects.rs
│   │       │   ├── fuel_log/
│   │       │   │   └── entity.rs
│   │       │   ├── expense/
│   │       │   │   └── entity.rs
│   │       │   ├── reminder/
│   │       │   │   └── entity.rs
│   │       │   ├── document/
│   │       │   │   └── entity.rs
│   │       │   ├── user/
│   │       │   │   ├── entity.rs
│   │       │   │   └── value_objects.rs    # Email, PasswordHash
│   │       │   └── errors.rs               # DomainError enum (thiserror)
│   │       ├── application/
│   │       │   ├── vehicle/
│   │       │   │   ├── create.rs
│   │       │   │   ├── list.rs
│   │       │   │   ├── get.rs
│   │       │   │   ├── update.rs
│   │       │   │   └── delete.rs
│   │       │   ├── service_record/
│   │       │   ├── fuel_log/
│   │       │   ├── expense/
│   │       │   ├── reminder/
│   │       │   ├── document/
│   │       │   └── auth/
│   │       │       ├── register.rs
│   │       │       └── login.rs
│   │       ├── ports/
│   │       │   ├── repositories.rs         # async traits: VehicleRepository, FuelLogRepository, …
│   │       │   ├── auth.rs                 # AuthPort trait
│   │       │   └── clock.rs                # Clock trait (SystemClock / MockClock for tests)
│   │       ├── adapters/
│   │       │   ├── inbound/
│   │       │   │   └── http/
│   │       │   │       ├── router.rs
│   │       │   │       ├── middleware/
│   │       │   │       │   ├── auth.rs     # JWT extraction → Extension<UserId>
│   │       │   │       │   └── tracing.rs
│   │       │   │       ├── handlers/
│   │       │   │       │   ├── health.rs
│   │       │   │       │   ├── auth.rs
│   │       │   │       │   ├── vehicles.rs
│   │       │   │       │   ├── service_records.rs
│   │       │   │       │   ├── fuel_logs.rs
│   │       │   │       │   ├── expenses.rs
│   │       │   │       │   ├── reminders.rs
│   │       │   │       │   └── documents.rs
│   │       │   │       └── dto/
│   │       │   │           ├── auth.rs
│   │       │   │           ├── vehicle.rs
│   │       │   │           ├── service_record.rs
│   │       │   │           ├── fuel_log.rs
│   │       │   │           ├── expense.rs
│   │       │   │           ├── reminder.rs
│   │       │   │           └── document.rs
│   │       │   └── outbound/
│   │       │       └── postgres/
│   │       │           ├── vehicle_repo.rs       # impl VehicleRepository
│   │       │           ├── service_record_repo.rs
│   │       │           ├── fuel_log_repo.rs
│   │       │           ├── expense_repo.rs
│   │       │           ├── reminder_repo.rs
│   │       │           ├── document_repo.rs
│   │       │           ├── user_repo.rs
│   │       │           └── models.rs             # sqlx row structs, distinct from domain entities
│   │       ├── bootstrap/
│   │       │   ├── config.rs                     # figment-based config (env + file)
│   │       │   └── state.rs                      # AppState { pool, jwt_secret, clock }
│   │       └── migrations/
│   │           ├── 001_users.sql
│   │           ├── 002_vehicles.sql
│   │           ├── 003_service_records.sql
│   │           ├── 004_fuel_logs.sql
│   │           ├── 005_expenses.sql
│   │           ├── 006_reminders.sql
│   │           └── 007_vehicle_documents.sql
│   └── frontend/
│       ├── package.json
│       ├── vite.config.ts
│       └── src/
│           ├── features/
│           │   ├── auth/
│           │   ├── vehicles/
│           │   └── dashboard/
│           └── shared/
├── packages/
│   └── openapi/
│       └── veyra.yaml                            # OpenAPI 3.1 contract
├── docker-compose.yml
├── docker-compose.dev.yml
├── Makefile
├── .github/
│   └── workflows/
│       ├── ci.yml
│       └── check-boundaries.sh                   # CI-enforced hexagonal layer guard
├── README.md
├── LICENSE
└── CHANGELOG.md
```

---

## Database Schema

### users

```sql
CREATE TABLE users (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email         TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    name          TEXT NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### vehicles

```sql
CREATE TABLE vehicles (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id          UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    brand            TEXT NOT NULL,
    model            TEXT NOT NULL,
    year             SMALLINT NOT NULL,
    plate_number     TEXT NOT NULL,
    color            TEXT,
    fuel_type        TEXT NOT NULL,  -- 'petrol' | 'diesel' | 'electric' | 'hybrid'
    current_odometer INTEGER NOT NULL DEFAULT 0,
    notes            TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, plate_number)
);
```

### service_records

```sql
CREATE TABLE service_records (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vehicle_id   UUID NOT NULL REFERENCES vehicles(id) ON DELETE CASCADE,
    service_date DATE NOT NULL,
    odometer     INTEGER NOT NULL,
    description  TEXT NOT NULL,
    workshop     TEXT,
    cost         NUMERIC(12,2),
    notes        TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### fuel_logs

```sql
CREATE TABLE fuel_logs (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vehicle_id      UUID NOT NULL REFERENCES vehicles(id) ON DELETE CASCADE,
    log_date        DATE NOT NULL,
    odometer        INTEGER NOT NULL,
    liters          NUMERIC(8,2) NOT NULL,
    price_per_liter NUMERIC(8,2) NOT NULL,
    total_cost      NUMERIC(12,2) GENERATED ALWAYS AS (liters * price_per_liter) STORED,
    station         TEXT,
    is_full_tank    BOOLEAN NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### expenses

```sql
CREATE TABLE expenses (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vehicle_id   UUID NOT NULL REFERENCES vehicles(id) ON DELETE CASCADE,
    expense_date DATE NOT NULL,
    category     TEXT NOT NULL,  -- 'tire' | 'battery' | 'tax' | 'insurance' | 'other'
    description  TEXT NOT NULL,
    amount       NUMERIC(12,2) NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### reminders

```sql
CREATE TABLE reminders (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vehicle_id    UUID NOT NULL REFERENCES vehicles(id) ON DELETE CASCADE,
    title         TEXT NOT NULL,
    reminder_type TEXT NOT NULL,  -- 'date' | 'odometer' | 'both'
    due_date      DATE,
    due_odometer  INTEGER,
    is_completed  BOOLEAN NOT NULL DEFAULT FALSE,
    notes         TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

> **Constraint (application layer):** `reminder_type = 'date'` requires `due_date NOT NULL`; `'odometer'` requires `due_odometer NOT NULL`; `'both'` requires both. Enforced in `application/reminder/create.rs`, not as a DB check constraint, to produce clean user-facing error messages.

### vehicle_documents

```sql
CREATE TABLE vehicle_documents (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vehicle_id  UUID NOT NULL REFERENCES vehicles(id) ON DELETE CASCADE,
    doc_type    TEXT NOT NULL,  -- 'stnk' | 'bpkb' | 'insurance' | 'other'
    title       TEXT NOT NULL,
    expiry_date DATE,
    file_url    TEXT,
    notes       TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### Indexes

```sql
CREATE INDEX idx_vehicles_user_id    ON vehicles(user_id);
CREATE INDEX idx_service_recs_vid    ON service_records(vehicle_id);
CREATE INDEX idx_fuel_logs_vid       ON fuel_logs(vehicle_id);
CREATE INDEX idx_expenses_vid        ON expenses(vehicle_id);
CREATE INDEX idx_reminders_vid_due   ON reminders(vehicle_id, due_date);
CREATE INDEX idx_documents_vid       ON vehicle_documents(vehicle_id);
```

---

## Error Handling

Three-layer error chain, all using `thiserror`:

```
domain/errors.rs       DomainError        (VehicleNotFound, InvalidOdometer, MissingDueDate, …)
application/*          AppError           (wraps DomainError; adds Unauthorized, Conflict, …)
adapters/inbound/http/ ApiError           (implements IntoResponse → JSON body + HTTP status)
main.rs                anyhow::Result<()> (binary boundary only)
```

Error response format (all endpoints):

```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "vehicle not found"
  }
}
```

Status code mapping in `ApiError::into_response`:

| Variant | HTTP Status |
|---------|------------|
| `NotFound` | 404 |
| `Unauthorized` | 401 |
| `Conflict` | 409 |
| `Validation` | 422 |
| `Internal` | 500 |

---

## Authentication

- **Algorithm:** JWT, HS256 (jsonwebtoken crate)
- **Payload:** `{ "sub": "<user_uuid>", "exp": <unix_ts> }`
- **Expiry:** 7 days (no refresh token in MVP)
- **Password hashing:** Argon2id (argon2 crate)

**Register flow:**
1. Validate email format + password minimum length (application layer)
2. Check email uniqueness (user repository)
3. Hash password with Argon2id
4. Insert user record
5. Sign and return JWT

**Login flow:**
1. Fetch user by email
2. Verify password with Argon2id
3. Sign and return JWT

**Protected route flow:**
1. `auth` middleware extracts `Authorization: Bearer <token>`
2. Verify JWT signature and expiry
3. Inject `Extension<UserId>` into the handler
4. Handler uses `UserId` to scope all database queries

---

## API Contract

```
GET    /health

POST   /auth/register
POST   /auth/login
GET    /me

GET    /vehicles
POST   /vehicles
GET    /vehicles/:id
PUT    /vehicles/:id
DELETE /vehicles/:id
GET    /vehicles/:id/summary          ← dashboard aggregation

GET    /vehicles/:id/services
POST   /vehicles/:id/services

GET    /vehicles/:id/fuel-logs
POST   /vehicles/:id/fuel-logs

GET    /vehicles/:id/expenses
POST   /vehicles/:id/expenses

GET    /vehicles/:id/reminders
POST   /vehicles/:id/reminders
PATCH  /vehicles/:id/reminders/:rid  ← mark complete / update

GET    /vehicles/:id/documents
POST   /vehicles/:id/documents
```

---

## Dashboard Summary Query

Single aggregation query — no N+1:

```sql
SELECT
    v.id,
    v.current_odometer,
    COUNT(DISTINCT s.id)                                         AS total_services,
    COALESCE(SUM(s.cost), 0)                                     AS total_service_cost,
    COUNT(DISTINCT f.id)                                         AS total_refuels,
    COALESCE(SUM(f.total_cost), 0)                               AS total_fuel_cost,
    COALESCE(SUM(e.amount), 0)                                   AS total_expenses,
    COUNT(DISTINCT r.id) FILTER (
        WHERE r.is_completed = FALSE
          AND (
            r.due_date     <= CURRENT_DATE + INTERVAL '30 days'
            OR r.due_odometer <= v.current_odometer + 500
          )
    )                                                            AS upcoming_reminders
FROM vehicles v
LEFT JOIN service_records s ON s.vehicle_id = v.id
LEFT JOIN fuel_logs       f ON f.vehicle_id = v.id
LEFT JOIN expenses        e ON e.vehicle_id = v.id
LEFT JOIN reminders       r ON r.vehicle_id = v.id
WHERE v.id = $1
  AND v.user_id = $2
GROUP BY v.id, v.current_odometer
```

---

## GitHub & README Design

### Repository Description (140 chars)

```
Veyra — open-source vehicle management API built with Rust. Track services, fuel, expenses & maintenance — self-hosted.
```

### Topics

```
rust  axum  postgresql  vehicle-management  rest-api  self-hosted  open-source  tokio  sqlx  fleet
```

### README Structure

1. **ASCII banner** — "VEYRA" art + tagline
2. **Badges row** — Rust version, License (MIT), CI status, PRs Welcome, Docker Ready (shields.io flat-square)
3. **One-liner** — "Track your vehicles, services, fuel, and expenses — all in one clean API."
4. **Architecture diagram** — Mermaid hexagonal layers (renders natively on GitHub)
5. **Features list** — emoji-prefixed, 8 items, no prose
6. **Quick start** — 5 steps, API working in under 60 seconds via Docker Compose
7. **API overview** — endpoint table
8. **Tech stack** — two-column table
9. **Roadmap** — GitHub checkbox list (v0.1 → v1.0)
10. **Contributing** — link to CONTRIBUTING.md
11. **License** — MIT

### Quick Start Target (5 steps)

```bash
git clone https://github.com/oksasatya/veyra && cd veyra
cp apps/backend/.env.example apps/backend/.env
docker compose up -d
docker compose exec backend cargo run --bin migrate
curl http://localhost:3000/health
# {"status":"ok","version":"0.1.0"}
```

---

## Rust Quality Gates

All code must pass the following before merge:

```
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run
cargo llvm-cov                         # ≥90% coverage on new code
cargo audit
```

Key non-negotiables:

- `#![forbid(unsafe_code)]` crate-wide
- No `.unwrap()` / `.expect()` / `panic!` on production code paths — use `Result` + `?`
- `thiserror` for domain and application error enums
- `anyhow::Result<()>` at `main.rs` binary boundary only
- `clippy::too_many_arguments` — ≤7 params (target ≤5); extract a params struct at 5+
- `clippy::cognitive_complexity` — ≤15; extract named helpers

---

## Development Roadmap

| Version | Scope |
|---------|-------|
| v0.1 | Monorepo scaffolding, Docker Compose, health check endpoint, CI pipeline |
| v0.2 | Auth: register, login, /me, JWT middleware |
| v0.3 | Vehicle CRUD |
| v0.4 | Service history |
| v0.5 | Fuel log + expense log |
| v0.6 | Maintenance reminders |
| v0.7 | Dashboard summary aggregation |
| v0.8 | React frontend (basic views) |
| v0.9 | OpenAPI 3.1 documentation |
| v1.0 | Stable MVP |

**Post-v1.0 ideas:** Tauri desktop app, Flutter mobile, OBD-II integration, service cost analytics, STNK/tax renewal reminders, two-crate workspace extraction.
