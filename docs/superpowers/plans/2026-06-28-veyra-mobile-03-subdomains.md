# Veyra Mobile — Plan 3: Sub-domain features (fuel, service, expense, reminder, document)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development. One subagent per feature (Tasks 1–5 are independent and parallelizable); the parent wires them into the detail tabs (Task 6).

**Goal:** Add the five vehicle sub-domain features so the detail screen's tabs and the "Log fuel" action work end-to-end: fuel logs, service records, expenses, reminders (with mark-complete), and documents — each a hexagonal feature (domain → data → presentation) that lists and creates entries scoped to a vehicle.

**Architecture:** Mirror the existing `lib/features/vehicle/` feature EXACTLY (domain entity + value objects, repository port, use cases, hand-written DTO, remote datasource, repo impl + Riverpod providers, presentation list widget + add bottom-sheet). Each feature lives only under `lib/features/<name>/` and edits NO shared files — the parent wires them into the detail tabs afterward.

**Tech Stack:** Flutter 3.x · Dart 3.x · flutter_riverpod 3 · dio · fpdart · decimal · go_router · mocktail. NO codegen (hand-written DTOs).

## Global Constraints (every task)

- **Template:** copy the structure + idioms of `lib/features/vehicle/` (entity, value_objects, repositories port, usecases, data/models DTO, data/datasources, data/repositories impl + providers, presentation). Do not invent new patterns.
- **Riverpod 3 gotchas (this repo's resolved versions):** use `AsyncNotifier` / `AsyncNotifierProvider` or `FutureProvider.family`; **`AsyncValue` has no `valueOrNull` — use `.asData?.value`**; `.isLoading` + `.when(...)` are fine.
- **Errors:** use cases return `Either<Failure, T>` (fpdart). `Failure` already `implements Exception` → a controller may `throw failure` into an `AsyncValue` error. Map `DioException` → `Failure` via `core/error/dio_error_mapper.dart` (`mapDioError`) in the repo impl (try/catch around datasource calls).
- **Networking:** use `ref.watch(dioProvider)` (from `core/network/dio_client.dart`). The interceptors already attach `Authorization: Bearer` + handle refresh — datasources just call `_dio.get/post/patch`.
- **Money:** monetary fields arrive as **strings** → parse to `Decimal` (`package:decimal`) in the DTO `toDomain()`. Never `double` for money.
- **DTOs:** hand-written `fromJson` (no json_serializable). List endpoints return a wrapper object (key per feature, below). Map DTO → domain entity via `toDomain()`.
- **Domain purity (CI-checked by `tool/check_boundaries.dart`):** files under any `domain/` import NO `package:flutter`, `package:dio`, `dart:io`. fpdart + decimal + core/error are OK.
- **Dart quality gate:** `flutter analyze` clean under `very_good_analysis`; const constructors; private widget constructors `const _Foo();`; no `print`; trailing-comma/format via `dart format`.
- **TDD verdict:** `TDD: yes` for value objects + any use case with branching (e.g. reminder cross-field rule) — write the test first. List/create pass-through use cases + DTO mappers → test-after (a mapper test is enough). Widgets → verify by analyze + the parent's screenshot.
- **Isolation rule (hard):** create ONLY files under your feature's `lib/features/<name>/` + a test under `test/features/<name>/`. Do NOT edit `app_router.dart`, `vehicle_detail_screen.dart`, `garage_screen.dart`, or any other feature. The parent wires you in (Task 6).
- Verify before done: `flutter analyze` clean + `flutter test` green for your feature.

## Shared API facts (per feature)

| Feature | List GET (wrapper key) | Create POST | Extra |
|---|---|---|---|
| fuel_log | `/vehicles/{id}/fuel-logs` → `{ "logs": [...] }` | same path | — |
| service_record | `/vehicles/{id}/services` → `{ "records": [...] }` | same path | — |
| expense | `/vehicles/{id}/expenses` → `{ "expenses": [...] }` | same path | — |
| reminder | `/vehicles/{id}/reminders` → `{ "reminders": [...] }` | same path | PATCH `/vehicles/{id}/reminders/{rid}` |
| document | `/vehicles/{id}/documents` → `{ "documents": [...] }` | same path | — |

Each list/create controller is keyed by `vehicleId` (use `AsyncNotifier`… or `FutureProvider.family` for read + a separate create call). Recommended: an `AsyncNotifierProvider.family<Controller, List<Entity>, String>` (vehicleId arg) with `refresh()` + `add(...)`, mirroring `GarageController` but family-keyed. (If family AsyncNotifier is awkward, use `FutureProvider.family` for the list + a plain create use case invoked from the sheet, then `ref.invalidate`.)

---

### Task 1: Fuel log feature

**TDD:** yes for `Liters`/`PricePerLiter` value objects (positive Decimal); mapper test for the DTO.

**Files (all under `lib/features/fuel_log/`):**
- `domain/entities/fuel_log.dart` — `FuelLog { id, vehicleId, logDate (DateTime), odometer (int), liters (Decimal), pricePerLiter (Decimal), totalCost (Decimal), station (String?), isFullTank (bool) }`
- `domain/value_objects/positive_decimal.dart` — optional helper `PositiveDecimal.create(String) -> Either<ValidationFailure, Decimal>` (reject ≤0 / unparseable), reused for liters + price.
- `domain/repositories/fuel_log_repository.dart` — port `list(vehicleId)`, `create(CreateFuelLogInput)`; `CreateFuelLogInput { vehicleId, logDate, odometer, liters (Decimal), pricePerLiter (Decimal), station?, isFullTank }`.
- `domain/usecases/{list_fuel_logs,create_fuel_log}_usecase.dart`
- `data/models/fuel_log_dto.dart` — `fromJson` keys: `id, vehicle_id, log_date, odometer, liters, price_per_liter, total_cost, station, is_full_tank`; `liters/price_per_liter/total_cost` are Strings → `Decimal.parse`; `log_date` ISO date → `DateTime.parse`.
- `data/datasources/fuel_log_remote_data_source.dart` — `list(vehicleId)` → GET, parse `data['logs']`; `create(vehicleId, body)` → POST.
- `data/repositories/fuel_log_repository_impl.dart` — impl + providers `fuelLogRepositoryProvider`, `listFuelLogsUseCaseProvider`, `createFuelLogUseCaseProvider`, and `fuelLogListProvider` (family by vehicleId).
- `presentation/widgets/fuel_log_list.dart` — a widget `FuelLogList({required String vehicleId})` rendering the list (date · odometer · liters L · station, trailing `Rp total_cost`) with loading/empty/error states. Reuse brand tokens from `core/theme/app_theme.dart`.
- `presentation/widgets/add_fuel_log_sheet.dart` — a bottom-sheet form (`showModalBottomSheet`) `AddFuelLogSheet({required String vehicleId})`: date (default today), odometer, liters, price/liter, station (optional), full-tank toggle; computes + shows total (liters × price) live; validates via VOs; calls create; pops on success. Match `design-system/screens/add-fuel-sheet.html`.
- `test/features/fuel_log/...` — VO tests + DTO mapper test.

**Create body JSON** (POST): `{ log_date, odometer, liters, price_per_liter, station?, is_full_tank }`. `liters`/`price_per_liter` sent as strings (Decimal.toString()) or numbers — send as strings to preserve precision.

---

### Task 2: Service record feature

**TDD:** mapper test; cost is optional Decimal.

**Files under `lib/features/service_record/`** (mirror Task 1):
- `domain/entities/service_record.dart` — `ServiceRecord { id, vehicleId, serviceDate (DateTime), odometer (int), description (String), workshop (String?), cost (Decimal?), notes (String?) }`
- port `list(vehicleId)` + `create(CreateServiceRecordInput { vehicleId, serviceDate, odometer, description, workshop?, cost (Decimal?), notes? })`; use cases; DTO (`records` wrapper; keys `service_date, odometer, description, workshop, cost (String?), notes`; `cost` nullable string → `Decimal?`); datasource; repo impl + providers + family list provider.
- `presentation/widgets/service_record_list.dart` (date · odometer · description · workshop, trailing `Rp cost` when present) + `add_service_record_sheet.dart` (date, odometer, description, workshop?, cost?, notes?).
- tests: DTO mapper.

---

### Task 3: Expense feature

**TDD:** yes for `ExpenseCategory` enum mapping; mapper test.

**Files under `lib/features/expense/`:**
- `domain/value_objects/expense_category.dart` — `enum ExpenseCategory { tire, battery, tax, insurance, other }` with `fromApi`, `apiValue` (= name), `label`.
- `domain/entities/expense.dart` — `Expense { id, vehicleId, expenseDate (DateTime), category (ExpenseCategory), description (String), amount (Decimal) }`
- port + use cases + DTO (`expenses` wrapper; keys `expense_date, category, amount (String), description`) + datasource + repo impl + providers + family list provider.
- `presentation/widgets/expense_list.dart` (category icon/label · date · description, trailing `Rp amount`) + `add_expense_sheet.dart` (date, category chips, description, amount).
- tests: category + mapper.

---

### Task 4: Reminder feature

**TDD:** yes — the cross-field rule (`reminder_type='date'` ⇒ dueDate required; `'odometer'` ⇒ dueOdometer required; `'both'` ⇒ both). Test the validator first.

**Files under `lib/features/reminder/`:**
- `domain/value_objects/reminder_type.dart` — `enum ReminderType { date, odometer, both }` + `fromApi`/`apiValue`/`label`.
- `domain/entities/reminder.dart` — `Reminder { id, vehicleId, title, type (ReminderType), dueDate (DateTime?), dueOdometer (int?), isCompleted (bool), notes (String?) }`
- `domain/usecases/create_reminder_usecase.dart` — enforces the cross-field rule, returning `Either<ValidationFailure, …>` before hitting the repo (or validate in the sheet via a domain validator function — put the rule in domain, tested).
- port `list(vehicleId)`, `create(CreateReminderInput)`, `complete(vehicleId, reminderId)` (PATCH `is_completed=true`); use cases incl. `CompleteReminderUseCase`.
- DTO (`reminders` wrapper; keys `title, reminder_type, due_date?, due_odometer?, is_completed, notes`) + datasource (list, create, `complete` via PATCH body `{ "is_completed": true }`) + repo impl + providers + family list provider.
- `presentation/widgets/reminder_list.dart` — grouped or flat; each row: title · vehicle/type meta · due pill; a check control → calls complete → refresh. Match `design-system/screens/reminders.html`.
- `presentation/widgets/add_reminder_sheet.dart` — title, type chips, conditional dueDate / dueOdometer fields (show per type), notes.
- tests: cross-field validator + DTO mapper.

---

### Task 5: Document feature

**TDD:** mapper test; `DocType` enum.

**Files under `lib/features/document/`:**
- `domain/value_objects/doc_type.dart` — `enum DocType { stnk, bpkb, insurance, other }` + `fromApi`/`apiValue`/`label`.
- `domain/entities/document.dart` — `Document { id, vehicleId, docType (DocType), title (String), expiryDate (DateTime?), fileUrl (String?) }`
- port + use cases + DTO (`documents` wrapper; keys `doc_type, title, expiry_date?, file_url?`) + datasource + repo impl + providers + family list provider.
- `presentation/widgets/document_list.dart` (doc icon · title · expiry; expiry status pill: expired / expiring-soon / valid / on-file when no expiry) + `add_document_sheet.dart` (doc type chips, title, expiry date?, file URL?, notes?). Match `design-system/screens/documents.html`.
- tests: doc-type + mapper + expiry-status helper.

---

### Task 6: Wire features into the detail tabs (PARENT — sequential, after Tasks 1–5)

**TDD:** no — composition; verify by `flutter analyze` + a screenshot.

- Modify `lib/features/vehicle/presentation/screens/vehicle_detail_screen.dart`:
  - Make the tab row stateful (selected index) and render the matching list widget below it per tab: Overview (current summary stays) · Fuel (`FuelLogList(vehicleId)`) · Service (`ServiceRecordList(vehicleId)`) · Expenses (`ExpenseList(vehicleId)`) · Docs (`DocumentList(vehicleId)`).
  - Wire the bottom "Log fuel" button → `showModalBottomSheet` with `AddFuelLogSheet(vehicleId)`; on the Service/Expenses/Docs tab, the button label/action switches to the matching add-sheet (or add a per-tab add affordance).
- `flutter analyze` clean, `flutter test` green, then the parent screenshots the detail tabs.

---

## Self-Review

- Coverage: 5 sub-domains (fuel/service/expense/reminder/document) each list + create; reminder adds complete; wired into detail tabs (Task 6). Matches the mobile design spec's domain table + the detail mockup tabs.
- Isolation: Tasks 1–5 touch only their own `features/<name>/` + tests → safe to run as parallel subagents. Task 6 (shared-file wiring) is parent-only, sequential, after all five land.
- Money via Decimal from string fields; Riverpod-3 `.asData?.value`; `Failure implements Exception`; hand DTOs; domain purity enforced by the boundary tool.
