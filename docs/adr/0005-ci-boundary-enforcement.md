# ADR-0005: CI Script for Hexagonal Boundary Enforcement

**Date:** 2026-06-27
**Status:** Accepted
**Deciders:** Oksa Satya

---

## Context

Hexagonal architecture's value depends on the domain layer remaining free of framework dependencies. Without enforcement, boundaries erode: a handler calls a repository directly, a domain entity gains a `sqlx::Type` derive, a use case imports an Axum extractor. This "architecture drift" is common in solo projects.

Two enforcement mechanisms were evaluated:

1. **Compiler enforcement via multi-crate workspace** — separate Cargo crates whose `Cargo.toml` dependency lists prevent illegal imports at compile time
2. **CI script enforcement** — a shell script that greps for forbidden import patterns and fails the build if found

Multi-crate workspace enforcement was rejected in ADR-0001 due to Rust's orphan rule making trait implementations for cross-crate types painful during the domain modeling phase.

---

## Decision

Use a **CI grep script** (`ci/check-boundaries.sh`) that runs on every pull request and push to `main`.

The script checks:

```bash
#!/usr/bin/env bash
set -euo pipefail

DOMAIN="apps/backend/src/domain"
APP="apps/backend/src/application"
PORTS="apps/backend/src/ports"

FORBIDDEN_IN_DOMAIN=("axum::" "sqlx::" "serde::" "tokio::")
FORBIDDEN_IN_APP=("axum::" "sqlx::")
FORBIDDEN_IN_PORTS=("axum::" "sqlx::")

check_layer() {
    local layer_path="$1"
    local layer_name="$2"
    shift 2
    local forbidden=("$@")

    for pattern in "${forbidden[@]}"; do
        if grep -r --include="*.rs" "$pattern" "$layer_path" 2>/dev/null | grep -v "^Binary"; then
            echo "ERROR: '$pattern' found in $layer_name layer — hexagonal boundary violated"
            exit 1
        fi
    done
}

check_layer "$DOMAIN" "domain"    "${FORBIDDEN_IN_DOMAIN[@]}"
check_layer "$APP"    "application" "${FORBIDDEN_IN_APP[@]}"
check_layer "$PORTS"  "ports"     "${FORBIDDEN_IN_PORTS[@]}"

echo "Boundary check passed."
```

---

## Consequences

### Positive

- Simple, transparent, readable — any developer can understand the rule by reading the script
- Fast CI feedback on boundary violations before code review
- Zero impact on build complexity (no workspace, no multi-crate setup)
- Can be run locally: `bash ci/check-boundaries.sh`

### Negative / Trade-offs

- Not compiler-enforced. A developer could modify the script to bypass it (though this would be visible in the PR diff)
- Does not catch all forms of coupling (e.g., naming conventions, structural coupling through shared global state)
- `serde` exclusion from `domain/` means domain entities cannot derive `Serialize`/`Deserialize` directly — DTOs in `adapters/inbound/http/dto/` handle serialization and map to/from domain types

### Note on serde in domain

The rule forbidding `serde` in `domain/` is a deliberate design choice. Domain entities are not JSON — they are typed Rust values. The DTOs in `adapters/inbound/http/dto/` derive `serde::Deserialize` (for request parsing) and `serde::Serialize` (for response serialization), and the adapter layer converts between DTOs and domain types. This mapping is intentional and keeps the domain pure.
