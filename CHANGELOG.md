# Changelog

All notable changes to Veyra will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

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
