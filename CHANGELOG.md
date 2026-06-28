# Changelog

All notable changes to Veyra will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added
- Redis-backed auth — rotating refresh tokens, session store with reuse detection, and read caching ([ADR-0006](docs/adr/0006-refresh-tokens-redis-sessions-cache.md))
- Dual-mode auth — bearer tokens for the native mobile client alongside the cookie + CSRF flow for browsers ([ADR-0007](docs/adr/0007-dual-mode-auth-bearer-mobile.md))
- Standardized response envelope — `{ meta, data | error }` with machine-readable error codes ([ADR-0008](docs/adr/0008-response-envelope-error-codes-i18n.md))
- i18n contract — `users.preferred_language` + `PATCH /me`; the API returns stable error codes and clients localize them
- Flutter mobile app (iOS + Android) in `apps/mobile/` — auth, garage, vehicle detail, service records, fuel logs, expenses, reminders, and documents, consuming the response envelope
- Mobile i18n foundation — `flutter_localizations` + ARB (en/id) + localized error mapping (`localizedFailure`) + device/override `LocaleController`
- CI — coverage gate via cargo-llvm-cov → Codecov (≥90%) and cargo-deny (advisories · licenses · sources)

### Changed
- Errors carry a stable `code`; collection responses return a bare array under `data`
- Declared `license = "MIT"` in the backend crate (`Cargo.toml`)
- Adopted Codecov + cargo-deny as the quality gate instead of SonarQube (no Rust analyzer)

## [0.1.0] - 2026-06-27

### Added
- Project scaffolding — hexagonal DDD layout, health endpoint
- Database schema — 7 tables (users, vehicles, service_records, fuel_logs, expenses, reminders, documents)
- Domain layer — entities, value objects, `DomainError`
- Ports — repository traits, `AuthPort`, `Clock`
- Auth — JWT (HS256, 7-day), Argon2id password hashing, register/login/me
- Vehicle CRUD — full ownership-scoped CRUD
- Service records — service history with cost tracking
- Fuel logs — consumption tracking with generated `total_cost` column
- Expenses — categorized expense tracking
- Reminders — date/odometer triggers with PATCH mark-complete
- Documents — expiry-tracked document store
- Dashboard — per-vehicle summary via CTE aggregation
- CI — GitHub Actions with hexagonal boundary enforcement script
- Dockerfile — multi-stage build (rust:1.82-slim-bookworm → debian:bookworm-slim)
