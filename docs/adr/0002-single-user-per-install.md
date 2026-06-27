# ADR-0002: Single-User Per Install (No Multi-Tenancy)

**Date:** 2026-06-27
**Status:** Accepted
**Deciders:** Oksa Satya

---

## Context

Vehicle management applications can be architected along a spectrum:

1. **Single-user per install** — one person deploys the binary for their own use; one user account in the database
2. **Multi-user, single database** — multiple people register on one server; rows scoped by `user_id` FK
3. **Multi-tenant SaaS** — multiple organizations/garages on one server; full tenant isolation via Postgres RLS or schema-per-tenant

The primary use cases for Veyra are personal vehicle owners, small rental businesses, and small workshops — all of which can self-host a single instance.

The MVP is also a portfolio project. Multi-tenancy adds significant complexity (RLS policies, tenant context propagation, AssertNoLeak integration tests) that would slow down shipping the core features.

---

## Decision

Veyra MVP is **single-user per install**.

One user registers on a deployed instance. The `users` table still exists (the auth system needs it for password hashing and JWT issuance), and all data tables carry a `user_id` FK for referential integrity — but the system is designed and operated as a single-user tool.

This means:
- No tenant isolation, no RLS, no `SET LOCAL app.current_tenant_id`
- The `user_id` scope on queries exists for integrity, not isolation
- No "invite user" or "team" feature in MVP

---

## Consequences

### Positive

- Significantly simpler domain model and queries
- No RLS policies to test or debug
- Faster MVP delivery
- Self-hosting story is extremely simple: `docker compose up`
- Clear and honest documentation for users

### Negative / Trade-offs

- Not suitable as a hosted SaaS without a significant architecture revision
- Users who want to share a vehicle fleet with family or a small team need separate instances

### Future action

If multi-user support is added later, the `user_id` FK already present on all data tables provides a foundation. Full multi-tenant SaaS would require Postgres RLS policies and tenant context propagation via a middleware-injected transaction wrapper — a significant but well-scoped change.
