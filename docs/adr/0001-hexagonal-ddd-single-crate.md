# ADR-0001: Hexagonal DDD Architecture — Single-Crate with CI-Enforced Boundaries

**Date:** 2026-06-27
**Status:** Accepted
**Deciders:** Oksa Satya

---

## Context

Veyra needs a backend architecture that:
1. Demonstrates real software engineering skill in a public portfolio repository
2. Keeps the domain logic independent of framework details (Axum, SQLx)
3. Remains learnable and maintainable for a solo developer learning Rust

Three structural approaches were evaluated:

- **A — Enhanced single-crate hexagonal DDD** (this decision): one `Cargo.toml`, layers separated by module paths, boundaries enforced via CI grep script
- **B — Multi-crate Cargo workspace from day one**: four separate crates (`veyra-domain`, `veyra-app`, `veyra-infra`, `veyra-api`) with compiler-enforced isolation
- **C — Vertical feature slices**: modules organized by feature (vehicles/, fuel/, services/) with internal layering

Approach B was evaluated through a cross-model debate (Claude + Codex GPT-5.5). Codex identified concrete problems:
- **Orphan rule trap**: you cannot implement `sqlx::Type`, `serde::Serialize`, or `axum::IntoResponse` for domain types defined in a separate crate without either polluting the domain crate with framework derives or writing large mapper/DTO layers
- **Visibility tax**: all cross-crate types must be `pub`, requiring reasoning about re-exports before the domain model is stable
- **Async trait pain**: async repository traits across crate boundaries force `async_trait`, boxed futures, and `Send + Sync` bounds, introducing significant noise before the API is running
- Portfolio signal comes from clean implementation, not from crate count

Approach C was rejected as the weakest option: vertical slices blur the hexagonal direction, cross-feature dependencies are awkward, and `shared/` tends to become a junk drawer.

---

## Decision

Adopt **Approach A**: single `Cargo.toml`, explicit layer folders, `ports/` as a first-class module (distinct from both `domain/` and `adapters/`), and CI enforcement via `ci/check-boundaries.sh`.

**Layer structure:**

```
src/
├── domain/           # entities, value objects, domain errors — zero external deps
├── application/      # use cases — imports domain + ports only
├── ports/            # repository/auth/clock traits — imports domain only
├── adapters/
│   ├── inbound/http/ # axum handlers, DTOs, middleware
│   └── outbound/postgres/ # sqlx repository implementations
└── bootstrap/        # AppState, wiring, composition root
```

**CI enforcement (`ci/check-boundaries.sh`):**

The CI script greps for forbidden import patterns and fails if found:
- `domain/` must not import `axum::`, `sqlx::`, `serde`, `tokio`
- `application/` must not import `axum::`, `sqlx::`
- `ports/` must not import `axum::`, `sqlx::`

This enforces the hexagonal contract without the multi-crate overhead.

---

## Consequences

### Positive

- No orphan rule conflicts — trait implementations for domain types live in the same crate
- Simpler build: one `Cargo.toml`, no workspace resolver concerns
- Faster iteration during domain model stabilization
- Hexagonal boundaries are real and enforced, just by tooling rather than the compiler
- Clear extraction path documented: when the domain is stable, extract to two crates (`veyra-core` + `veyra-adapters`)

### Negative / Trade-offs

- Convention + CI script, not compiler, enforces the boundary. A developer could bypass it by editing the script or skipping CI.
- Cannot independently version or publish individual layers as crates until extraction

### Future action

When the domain model stabilizes (after v0.7 dashboard summary), evaluate extraction to:
```
crates/
├── veyra-core/      # domain + application + ports
└── veyra-adapters/  # inbound/http + outbound/postgres
apps/backend/        # binary only
```

This two-crate shape (not four) avoids the orphan rule problem because `veyra-core` can carry `serde` derives if needed, and `veyra-adapters` handles all framework-specific code.
