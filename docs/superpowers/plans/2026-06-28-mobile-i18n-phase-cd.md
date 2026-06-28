# Plan — Mobile i18n Phase C (UI strings) + Phase D (language toggle + backend sync)

**Date:** 2026-06-28
**Depends on:** i18n foundation (commit `1f7b68f`) — `AppLocalizations` (en/id), `LocaleController`,
`localizedFailure`, `MaterialApp` wired with delegates + `locale`.
**Goal:** every user-facing string localized (en/id), errors shown via `localizedFailure`, and a
settings language toggle that drives `LocaleController` + syncs `PATCH /me { preferred_language }`.

---

## Conventions (apply in every UC)

- **ARB keys:** `camelCase`, grouped by area — `auth*`, `garage*`, `vehicle*`, `reminder*`,
  `document*`, `expense*`, `fuelLog*`, `service*`, and shared `common*` (e.g. `commonRetry`,
  `commonCancel`, `commonSave`, `commonDelete`, `commonLoading`, `commonEmpty`). Reuse `common*`
  aggressively — do NOT create per-screen duplicates of "Retry"/"Cancel"/"Save".
- **Add to BOTH** `lib/l10n/app_en.arb` and `lib/l10n/app_id.arb`, then run `flutter gen-l10n`.
  A key missing from `app_id.arb` is a (CI-visible) untranslated-message warning — keep them in sync.
- **Placeholders** for interpolated values (counts, names, dates): ARB `{ "placeholders": { ... } }`,
  e.g. `"garageVehicleCount": "{count} vehicles", "@garageVehicleCount": { "placeholders": { "count": { "type": "int" } } }`.
  Use `intl` plural/select syntax where a count drives wording.
- **Access:** `final l10n = AppLocalizations.of(context);` at the top of `build`; then `l10n.key`.
- **Errors:** replace every `failure.message` / `e is Failure ? e.message : '...'` display with
  `localizedFailure(l10n, failure)` (import `core/error/failure_l10n.dart`). The hardcoded English
  fallbacks (`'Could not load expenses.'` etc.) become `l10n.*` keys or a generic `l10n.errorServer`.
- **Lint:** keep `flutter analyze` (very_good_analysis) clean after every UC.
- **TDD verdict (§16): NO** for the string-extraction UCs (visual/string work — verify by
  `flutter analyze` + running the app + switching language). `localizedFailure` + `LocaleController`
  already have a clear contract → add a small unit test in UC-D1/UC-D5 where it adds signal.

---

## Phase C — per-screen string localization

One UC per feature. Each UC: (a) extract hardcoded strings → ARB (en+id) → `l10n.*`; (b) switch
that feature's error-display sites to `localizedFailure`. Run `flutter gen-l10n` + `flutter analyze`
after each. Ordered smallest-blast-radius first to validate the pattern early, biggest last.

- **UC-C1 — Auth** (`login_screen` 9, `register_screen` 7, `splash_screen` 2). Error sites:
  `login_screen:49`, `register_screen:51` (`_error = failure.message` → `localizedFailure`). Keys:
  `authLogin*`, `authRegister*`, `authEmailLabel`, `authPasswordLabel`, `authNameLabel`, etc.
- **UC-C2 — Vehicle** (`garage_screen` ~23, `vehicle_detail_screen` ~13, `add_vehicle_screen` 5).
  **Largest UC.** Error sites: `garage_screen:93`, `vehicle_detail_screen:129`, `add_vehicle_screen:83`.
  Keys: `garage*`, `vehicleDetail*`, `vehicleAdd*`, fuel-type labels (`fuelTypePetrol`, …).
- **UC-C3 — Reminder** (`reminders_overview` 8, `reminder_list` 7, `add_reminder_sheet` 6). Error
  sites: `reminders_overview:30,117` (SnackBar), `reminder_list:22,62` (SnackBar). Reminder-type labels.
- **UC-C4 — Document** (`documents_overview` 7, `document_list` 8, `add_document_sheet` 6). Error site
  in the overview/list. Doc-type labels (`docTypeStnk`, `docTypeInsurance`, …).
- **UC-C5 — Expense** (`expense_list` 6, `add_expense_sheet` 6). Error sites: `add_expense_sheet:74`,
  `expense_list:22`. Category labels.
- **UC-C6 — Fuel log** (`add_fuel_log_sheet` 9, `fuel_log_list` 6). Error site in the list.
- **UC-C7 — Service record** (`add_service_record_sheet` 5, `service_record_list` 6). Error site.
- **UC-C8 — Shared widgets** (`status_pill`, `segmented_tabs`): localize only if they hold their own
  literals; most labels are passed in by callers (already localized upstream) → likely no change.

**Note on value-object validation messages:** `features/*/domain/value_objects/*.dart` build
`ValidationFailure('Enter a plate number.', field: 'plate')` with English literals. These are
client-side validations (not backend codes). Options: (a) leave English for now, (b) move them to
`l10n.*` by passing a localized string in at the call site. **Decision: defer to a follow-up** —
they are client-origin, not part of the backend error-code contract; localizing them needs the
`BuildContext`/`l10n` at the validation call site (architectural). Track separately.

---

## Phase D — language toggle + backend sync

- **UC-D1 — `updatePreferences` data path.** Add `Future<void> updatePreferences(String language)` to
  the auth/user remote data source → `PATCH /me`, body `{ "preferred_language": language }` (backend
  ADR-0008). Thread through the auth repository + a `UpdatePreferences` use case (or a method on the
  existing auth controller). Map errors via the existing `mapDioError`. **TDD: light** — one test that
  the data source posts the right body to `/me` (mocktail dio).
- **UC-D2 — Settings screen** (`features/settings/presentation/screens/settings_screen.dart` + route
  `/settings`). A language selector: **System default / English / Indonesian** (radio/segmented),
  reflecting the current `localeControllerProvider` value (null = system). On change:
  `ref.read(localeControllerProvider.notifier).setLocale(locale)`. Localize its strings (`settingsTitle`,
  `settingsLanguage`, `languageSystem/English/Indonesian` — already in ARB).
- **UC-D3 — Backend sync.** When the user is authenticated and picks a concrete language (en/id), also
  call UC-D1 `updatePreferences(code)` so server-generated content (future notifications) matches.
  "System default" → send the resolved device language, or skip the sync (decide: **send resolved
  device code** so the server always has a concrete value). Fire-and-forget with error surfaced as a
  non-blocking SnackBar (`localizedFailure`); the local toggle still applies regardless of sync result.
- **UC-D4 — Entry point.** Add a settings icon to the garage app bar → `context.push('/settings')`.
- **UC-D5 — Precedence + adoption.** Local override wins for the UI locale. On login/`me`, if there is
  NO local override, adopt the backend `preferred_language` (so a returning user keeps their server
  choice). Requires `UserDto` to expose `preferred_language` (currently ignored) → add the field +
  on auth success, if `LocaleController` has no persisted override, `setLocale(Locale(serverCode))`.
  **TDD: light** — LocaleController precedence (override beats backend) unit test.

---

## Sequencing & verification

1. Phase C UCs in listed order (C1 validates the pattern; C2 is the bulk). Commit per 1–2 UCs.
2. Phase D after C (settings screen can reuse localized `common*` keys).
3. After each UC: `flutter gen-l10n` (if ARB changed) → `flutter analyze` (clean) → spot-run.
4. Final: run the app, toggle en↔id live (Riverpod rebuild), trigger a 422 (e.g. bad plate) → assert
   the localized `INVALID_PLATE_NUMBER` message; trigger a network error → `errorNetwork`.

## Out of scope (tracked follow-ups)

- Localizing client-side value-object validation messages (UC-C note).
- Date/number/currency formatting via `intl` (`DateFormat`/`NumberFormat`) per locale — separate pass.
- Pluralization audit across all count strings.
