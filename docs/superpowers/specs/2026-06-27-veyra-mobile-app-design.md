# Veyra Mobile App — Design Specification

**Date:** 2026-06-27
**Status:** Approved
**Author:** Oksa Satya
**Depends on:** ADR-0007 + `docs/superpowers/specs/2026-06-27-mobile-bearer-auth-design.md` (backend bearer mode)
**Consumes:** `packages/openapi/veyra.yaml` (REST contract)

---

## Overview

A cross-platform (iOS + Android, single codebase) Flutter client for the Veyra vehicle-management API.
It is a thin, well-structured consumer of the REST backend: authentication, vehicle CRUD, service
history, fuel logs, expenses, maintenance reminders, documents, and a per-vehicle dashboard summary.

The app is a **portfolio piece**, so it deliberately mirrors the backend's discipline:
**hexagonal architecture (ports & adapters) with DDD *tactical* building blocks** — entities, value
objects, repository ports, use cases. It does **not** apply DDD *strategic*/full ceremony (aggregate-root
enforcement, domain events, domain services, CQRS): the real business rules live in the Rust backend, so
those would be boilerplate without payoff on the client. This is the "Clean Architecture" Flutter shape.

The Swift iOS prototype and any desktop target are dropped; Flutter is the single client.

---

## Goals

- One Flutter codebase shipping to both iOS and Android.
- Mirror the backend's hexagonal + DDD-tactical discipline as a portfolio showcase.
- Strict layer isolation: the domain layer is pure Dart and imports no Flutter/dio/storage types.
- Idiomatic native auth: bearer tokens in secure storage, transparent refresh, route guarding.
- Cover every backend domain, building auth + vehicle first as the reusable feature template.

## Non-Goals

- Offline-first / local persistence cache (pull-to-refresh + in-memory only).
- Push notifications / background reminders (the backend has no push; reminders are read-only here).
- File/document upload (backend stores URLs only in MVP — the app shows/links them).
- DDD strategic patterns (aggregates as transactional roots, domain events, CQRS).
- Desktop / web Flutter targets.
- Certificate pinning, biometric lock (noted as future hardening, out of MVP scope).

---

## Tech Stack

| Concern | Choice | Notes |
|---|---|---|
| Framework | Flutter (stable 3.x) · Dart 3.x | records + patterns available |
| State / DI | **Riverpod** (`flutter_riverpod` + `riverpod_annotation`) | providers ARE the DI container; no get_it |
| Networking | **dio** | interceptors for auth, refresh, error mapping |
| Routing | **go_router** | declarative + auth redirect guard |
| Functional errors | **fpdart** (`Either<Failure, T>`) | use cases return `Either` |
| Immutable models | **freezed** + **json_serializable** | DTOs (data layer) get JSON; domain entities are freezed but JSON-free |
| Secure storage | **flutter_secure_storage** | access + refresh tokens (Keychain / Keystore) |
| Testing | `flutter_test` + **mocktail** | unit (domain), mapper, widget tests |
| Lint gate | **very_good_analysis** | strict analyzer ruleset |

---

## Architecture

### Layering (hexagonal + DDD tactical), per feature

Three layers per feature, plus a cross-cutting `core/`. Dependency direction is strictly inward —
`presentation → domain ← data`; **domain depends on nothing outward.**

| Layer | Contains | May import | Forbidden |
|---|---|---|---|
| `domain/` | entities, value objects, repository **interfaces** (ports), use cases, `Failure` | pure Dart, `fpdart` | `flutter/*`, `dio`, `json_*`, storage |
| `data/` | DTO models (`fromJson`/`toJson`), mappers, remote datasources, repository **impls** | `domain`, `dio`, `json_serializable` | `flutter/widgets` |
| `presentation/` | Riverpod controllers/notifiers, screens, widgets | `domain`, `flutter`, `riverpod` | `dio`, `data` impls directly |
| `core/` | theme, dio client, interceptors, router, DI roots, shared `Failure` base | all | — |

The **domain layer is the hexagon's core**; repository interfaces are the **ports**; the Postgres-backed
REST is reached through **adapters** (datasource + repo impl). Presentation talks to use cases, never to
dio or a repo impl directly — it depends on the port, resolved by Riverpod.

> **Boundary discipline (mirror of the backend's CI guard):** add an analyzer/import-lint rule (or a
> small `tool/check_boundaries.dart`) that fails if `domain/` imports `package:flutter`, `package:dio`,
> or `dart:io` — the client analogue of `ci/check-boundaries.sh`.

### Why this and not full DDD

DDD tactical blocks (entity, value object, repository port, use case) earn their keep on a client: they
make validation explicit and the data flow testable. DDD strategic blocks (aggregate roots as
consistency boundaries, domain events, domain services) model *server-side* invariants — replicating
them here duplicates rules the backend already owns. So the client keeps tactical, skips strategic.

---

## Folder Structure

`auth` and `vehicle` are shown in full as the templates; every other feature replicates the same shape.

```
apps/mobile/
├── pubspec.yaml
├── analysis_options.yaml            # very_good_analysis + domain-import boundary rule
├── tool/check_boundaries.dart       # fails build if domain/ imports flutter/dio/dart:io
├── lib/
│   ├── main.dart                    # ProviderScope + VeyraApp
│   ├── core/
│   │   ├── config/app_config.dart   # base URL (dart-define), TTL hints
│   │   ├── theme/                   # ColorScheme (brand), typography, theme data
│   │   ├── network/
│   │   │   ├── dio_client.dart      # dio factory + provider
│   │   │   ├── auth_interceptor.dart      # inject Authorization: Bearer + X-Auth-Mode
│   │   │   ├── refresh_interceptor.dart   # single-flight 401 → refresh → retry
│   │   │   └── error_interceptor.dart     # DioException → Failure
│   │   ├── error/failure.dart       # sealed Failure hierarchy
│   │   ├── router/app_router.dart   # go_router + auth redirect
│   │   └── storage/token_store.dart # flutter_secure_storage wrapper (+ provider)
│   └── features/
│       ├── auth/
│       │   ├── domain/
│       │   │   ├── entities/user.dart            # freezed, JSON-free
│       │   │   ├── value_objects/email.dart      # Email.create -> Either<ValidationFailure,Email>
│       │   │   ├── value_objects/password.dart
│       │   │   ├── repositories/auth_repository.dart   # PORT (abstract)
│       │   │   └── usecases/{login,register,logout,get_me}.dart
│       │   ├── data/
│       │   │   ├── models/{user_dto,auth_tokens_dto,auth_response_dto}.dart
│       │   │   ├── datasources/auth_remote_data_source.dart
│       │   │   └── repositories/auth_repository_impl.dart
│       │   └── presentation/
│       │       ├── controllers/auth_controller.dart   # Riverpod Notifier<AuthState>
│       │       ├── screens/{login,register}_screen.dart
│       │       └── widgets/
│       ├── vehicle/
│       │   ├── domain/  (Vehicle entity; PlateNumber/Odometer/FuelType VOs; VehicleRepository port;
│       │   │            list/get/create/update/delete usecases)
│       │   ├── data/    (VehicleDto + mapper; VehicleRemoteDataSource; VehicleRepositoryImpl)
│       │   └── presentation/ (VehicleListController, VehicleDetailController, screens, widgets)
│       ├── service_record/   # same 3-layer shape
│       ├── fuel_log/
│       ├── expense/
│       ├── reminder/
│       ├── document/
│       └── dashboard/        # consumes /vehicles/:id/summary
└── test/
    └── features/<f>/
        ├── domain/   # value-object + use-case tests (TDD)
        └── data/     # DTO mapper tests
```

---

## Domain Modeling (tactical)

Entities mirror the backend domain; value objects carry the validation the user can trigger client-side.

| Feature | Entity | Value objects (with `create() -> Either<ValidationFailure, T>`) |
|---|---|---|
| auth | `User { id, email, name }` | `Email`, `Password` (min length) |
| vehicle | `Vehicle { id, brand, model, year, plate, color?, fuelType, odometer, notes? }` | `PlateNumber` (non-empty, normalized), `Odometer` (u32 ≥ 0), `FuelType` (enum: petrol/diesel/electric/hybrid), `VehicleYear` |
| service_record | `ServiceRecord { id, vehicleId, date, odometer, description, workshop?, cost?, notes? }` | `Money` (cost), `Odometer` |
| fuel_log | `FuelLog { id, vehicleId, date, odometer, liters, pricePerLiter, totalCost, station?, isFullTank }` | `Liters`, `PricePerLiter`, `Money` |
| expense | `Expense { id, vehicleId, date, category, description, amount }` | `ExpenseCategory` (tire/battery/tax/insurance/other), `Money` |
| reminder | `Reminder { id, vehicleId, title, type, dueDate?, dueOdometer?, isCompleted, notes? }` | `ReminderType` (date/odometer/both) + cross-field rule: `date`→dueDate required, `odometer`→dueOdometer required, `both`→both |
| document | `Document { id, vehicleId, docType, title, expiryDate?, fileUrl?, notes? }` | `DocType` (stnk/bpkb/insurance/other) |
| dashboard | `VehicleSummary { vehicleId, currentOdometer, totalServices, totalServiceCost, totalRefuels, totalFuelCost, totalExpenses, upcomingReminders }` | — (read-only aggregate) |

Money uses `Decimal` semantics (the `decimal` package) to match the backend's `NUMERIC` — never `double`
for currency.

---

## Error Model

A sealed `Failure` hierarchy in `core/error/failure.dart`; use cases return `Either<Failure, T>`.

```
sealed Failure
 ├─ NetworkFailure          // no connectivity / timeout
 ├─ ServerFailure(message)  // 5xx or unexpected
 ├─ UnauthorizedFailure     // 401 after refresh also failed
 ├─ NotFoundFailure         // 404
 ├─ ConflictFailure(message)// 409 (e.g. duplicate plate)
 └─ ValidationFailure(field?, message)  // 422 + client-side VO validation
```

**Backend error parsing.** Per the bearer-auth spec, the live API error body is the flat shape
`{"error":"<message>"}` (from `AppError::into_response`). The `error_interceptor` maps
`DioException` → `Failure` by status code, reading `response.data["error"]` as the message string. Do
**not** assume a nested `{"error":{"code","message"}}` body — the implementation emits the flat form.

Presentation renders failures as snackbars (transient) or inline field errors (`ValidationFailure`).

---

## Networking & Auth Flow

### dio interceptors (order matters)

1. **AuthInterceptor** — on each request: attach `Authorization: Bearer <access>` from `TokenStore`;
   on the four auth endpoints attach `X-Auth-Mode: bearer`.
2. **RefreshInterceptor** — on `401`: perform a **single-flight** refresh (a shared `Completer`/lock so N
   concurrent 401s trigger exactly one `POST /auth/refresh`; the others await it), then retry the
   original requests with the new access token. If refresh fails → clear tokens, surface
   `UnauthorizedFailure`, and signal the auth controller to redirect to login.
3. **ErrorInterceptor** — map remaining `DioException`s to `Failure`.

### Token lifecycle

- Login/register (bearer) → store `access_token` + `refresh_token` in `flutter_secure_storage`.
- `refresh_token` wire format is the opaque `{family_id}.{raw_secret}` string — stored and replayed
  verbatim in the `{"refresh_token":"..."}` body; the client never parses it.
- Refresh **rotates** the refresh token (backend returns a new one) → the client must persist the new
  refresh token on every successful refresh.
- Logout → `POST /auth/logout` with the refresh body, then wipe secure storage.

### Route guarding

`go_router`'s `redirect` reads the Riverpod `AuthController` state: unauthenticated → `/login`;
authenticated hitting `/login` → `/` (vehicle list). The controller is the single source of auth truth.

---

## State Management (Riverpod)

- **DI providers** (in `core` / per feature): `dioProvider`, `tokenStoreProvider`, datasource providers,
  repository providers (the port type, bound to the impl — overridable in tests), use-case providers.
- **Controllers:** `AuthController` (`Notifier<AuthState>`); list screens use `AsyncNotifier` returning
  `Either`-unwrapped domain lists; mutations call the use case and refresh the relevant provider.
- **No business logic in widgets** — widgets watch a controller/provider and render; actions call
  controller methods.

---

## Screen / Navigation Map

```
/login                      LoginScreen
/register                   RegisterScreen
/                           VehicleListScreen        (+ dashboard summary card per vehicle)
/vehicles/new               VehicleFormScreen (create)
/vehicles/:id               VehicleDetailScreen      (tabs: Overview | Service | Fuel | Expense | Reminder | Document)
/vehicles/:id/edit          VehicleFormScreen (edit)
   per tab: list + "add" form sheet
   reminder tab: mark-complete (PATCH /vehicles/:id/reminders/:rid)
```

The detail screen's tabs each consume their feature's list use case scoped to `vehicleId`. The Overview
tab consumes `/vehicles/:id/summary`.

---

## Theming (brand)

Material 3, **dark-first**, driven by the Veyra brand (logo: orange V-monogram + lowercase wordmark):

- `ColorScheme.dark` seeded from accent `#F26A21` (logo orange), surfaces on graphite `#0D1119`,
  surface `#151A23`, text `#E6EAF0`, secondary text `#9BA6B5`.
- Display/wordmark font = geometric sans (matching the logo); body = Inter.
- Logo asset used on splash + app icon (iOS + Android via `flutter_launcher_icons`).
- Light theme is a stretch goal (the same accent works on a light surface); dark ships first.

Tokens are centralized in `core/theme/` as Dart constants (reference → semantic), mirroring the
design-token approach so a later `/design-sync` can map cleanly.

---

## API Contract Consumption

All endpoints from the main spec / `packages/openapi/veyra.yaml`. Auth endpoints send
`X-Auth-Mode: bearer`; protected endpoints send `Authorization: Bearer`. List endpoints return full
arrays (no pagination in the backend MVP) — the client renders the full list and offers pull-to-refresh.

| Feature | Endpoints |
|---|---|
| auth | `POST /auth/register`, `POST /auth/login`, `POST /auth/refresh`, `POST /auth/logout`, `GET /me` |
| vehicle | `GET/POST /vehicles`, `GET/PUT/DELETE /vehicles/:id` |
| service | `GET/POST /vehicles/:id/services` |
| fuel | `GET/POST /vehicles/:id/fuel-logs` |
| expense | `GET/POST /vehicles/:id/expenses` |
| reminder | `GET/POST /vehicles/:id/reminders`, `PATCH /vehicles/:id/reminders/:rid` |
| document | `GET/POST /vehicles/:id/documents` |
| dashboard | `GET /vehicles/:id/summary` |

---

## Dart Quality Gate

```
dart format --set-exit-if-changed .
flutter analyze            # very_good_analysis ruleset, zero issues
flutter test               # unit + mapper + widget tests
dart run tool/check_boundaries.dart   # domain/ imports no flutter/dio/dart:io
```

Key rules:
- Full null-safety; avoid `dynamic`; avoid `late` where a nullable + guard is clearer.
- No `print()` in app code — use a logger.
- Value objects validate in `create()`; constructors stay private — invalid instances are unrepresentable.
- Currency via `Decimal`, never `double`.
- Domain layer free of `package:flutter`, `package:dio`, `dart:io` (enforced by the boundary tool).

---

## Testing Strategy

`TDD: yes` for domain (value-object validation + use cases) and data mappers — clear input→output
contracts. `TDD: no` (test-after) for widgets/screens (verify by running + widget tests).

- **Domain unit tests:** each value object's `create()` accept/reject cases; each use case with a
  mocktail-mocked repository port (success + each `Failure`).
- **Data mapper tests:** DTO `fromJson` → entity mapping round-trips, including null/optional fields.
- **Widget tests:** login form (validation + submit), vehicle list (loading/empty/error/data states).
- **Auth interceptor test:** 401 → single-flight refresh → retry succeeds; refresh-fail → logout.

---

## Sequencing / Build Order

1. **Backend bearer mode** (prerequisite — `docs/superpowers/plans/2026-06-27-mobile-bearer-auth.md`).
2. **Scaffold + core:** project, theme, dio + interceptors, token store, router, `Failure`, boundary tool.
3. **Feature `auth`** — full hexagonal + DDD-tactical pass; becomes the **template** for every feature.
4. **Feature `vehicle`** — CRUD, replicating the auth template (list/detail/form).
5. **Dashboard summary** — per-vehicle aggregate card.
6. **service / fuel / expense / reminder / document** — replicate the template (parallelizable).
7. **Polish:** theming pass, app icon/splash, empty/error/loading states, widget tests.

---

## Open Decisions Deferred to the Plan

- Exact freezed vs hand-written model split (default: freezed for entities + DTOs).
- Whether the boundary check is an analyzer custom-lint or a standalone Dart script (default: script in
  `tool/`, fastest to ship).
- Light theme timing (default: after dark ships).
