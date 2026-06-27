# ADR-0004: SQLx for Database Access (over Diesel and SeaORM)

**Date:** 2026-06-27
**Status:** Accepted
**Deciders:** Oksa Satya

---

## Context

Three mature database access options were considered for the Rust backend:

1. **SQLx** — async-first, compile-time query verification, raw SQL, no ORM
2. **Diesel** — compile-time safe query builder DSL, sync-first (async via diesel-async), mature ecosystem
3. **SeaORM** — async ORM, entity-based, built on SQLx, higher-level abstraction

---

## Decision

Use **SQLx** directly.

---

## Rationale

**SQLx fits the project's goals best:**

- **Async-native**: SQLx is designed around `tokio`. Diesel's async story requires `diesel-async` as a separate crate, adding complexity.
- **Raw SQL**: Writing SQL directly in repository implementations is more educational for a portfolio project and produces queries that are easy to read and optimize. The dashboard summary query (multi-table aggregation with `FILTER`) is straightforward in raw SQL but requires significant DSL gymnastics in Diesel or SeaORM.
- **Compile-time query verification**: `sqlx::query!` and `sqlx::query_as!` macros verify SQL at compile time against a live database (or offline `.sqlx/` snapshot). This catches type mismatches early without a full ORM.
- **Domain separation**: In the hexagonal architecture, SQLx is exclusively in `adapters/outbound/postgres/`. The domain layer never sees SQLx types. This is easier to achieve with raw SQL + manual mapping than with an ORM that tends to bleed its types into calling code.
- **Migrations**: `sqlx-migrate` (or the `sqlx migrate` CLI) handles schema migrations in plain SQL files, which are readable by any developer regardless of Rust expertise.

**Why not Diesel:** The sync-first design and DSL-centric query building would make the dashboard aggregation query significantly harder to write and read. The async wrapper adds complexity without clear benefit.

**Why not SeaORM:** The ORM abstraction obscures what SQL is actually being executed, which is the opposite of what a portfolio project should demonstrate. Entity-generated queries for multi-table aggregations are also harder to control.

---

## Consequences

### Positive

- All SQL is visible and readable in the source code
- Async-native, no wrappers needed
- Compile-time query verification without full ORM overhead
- Migration files are plain `.sql` — reviewable without Rust knowledge
- Clear separation: SQLx types never appear outside `adapters/outbound/postgres/`

### Negative / Trade-offs

- More manual mapping between sqlx row structs and domain entities (handled in `models.rs` within the postgres adapter)
- No automatic query generation for simple CRUD — every query is hand-written
- Compile-time verification requires a running database or a pre-generated `.sqlx/` snapshot (managed via `SQLX_OFFLINE=true` in CI)
