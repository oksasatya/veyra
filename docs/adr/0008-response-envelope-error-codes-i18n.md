# ADR-0008: Standardized Response Envelope, Error Codes, and i18n Strategy

**Date:** 2026-06-28
**Status:** Accepted
**Relates to:** ADR-0007 (Dual-Mode Auth) — both auth delivery modes now return the envelope
**Deciders:** Oksa Satya (cross-model design pass with Codex / GPT-5.5)

> Establishes a single response shape for the whole API and the internationalization (i18n)
> contract between the backend and its clients. The center of gravity for localization is the
> **client**: it owns all UI copy and maps stable machine-readable error **codes** to localized
> messages. The backend localizes only content it generates without a client in the loop (future
> notifications/emails), driven by a stored per-user language.

---

## Context

Before this decision the API returned ad-hoc shapes: success bodies were bare resources
(`{ "brand": "Toyota" }`) or named-collection wrappers (`{ "vehicles": [...] }`), and errors were a
flat `{ "error": "<english prose>" }`. Two problems:

1. **No consistent envelope.** Each endpoint had its own shape, so a client could not write one
   response parser, one error handler, or a uniform place for cross-cutting metadata (a request id
   for support correlation, pagination later).
2. **No i18n contract.** Error responses were English prose. Veyra targets English **and**
   Indonesian and has a dedicated Flutter client (a web client may follow). Returning localized
   prose from the backend per `Accept-Language` couples the API to locale, risks two sources of
   truth (client copy *and* server copy drifting), and a raw server string is not a stable contract
   a client can key localized messages off.

A further forward-looking need: Veyra has **maintenance reminders**. When those become background
notifications, the server generates the user-facing text with no client in the loop — so the
backend must know the user's language for *that* content specifically.

### Alternatives considered

The design was stress-tested in a cross-model pass with Codex (GPT-5.5). Three coherent shapes:

1. **Idiomatic: bare-resource success + RFC 9457 `application/problem+json` errors.** Most
   "standards-flex"; but success has no envelope for metadata, and mixing two content types was
   more ceremony than this project wants.
2. **Full custom envelope `{ meta, data | error }` for every response** *(chosen)* — one shape, one
   client parser, a natural home for `request_id` (and pagination later).
3. **Keep flat shapes, add an `X-Error-Code` header** — minimal change, but leaves success
   unstructured and hides the code off-body.

On i18n: returning **stable codes** (client localizes) vs **localized prose** (`Accept-Language`).
Codes won — they keep the contract stable and localization single-sourced in the client.

---

## Decision

### 1. Response envelope (every JSON response)

Success — single resource (`application/json`, HTTP status carries 200/201):

```json
{ "meta": { "request_id": "..." }, "data": { ...resource... } }
```

Success — collection (the array is **directly** at `data`; no wrapper key):

```json
{ "meta": { "request_id": "..." }, "data": [ ... ] }
```

Error:

```json
{ "meta": { "request_id": "..." }, "error": { "code": "INVALID_PLATE_NUMBER", "message": "invalid plate number: ABC" } }
```

- `data` and `error` are **mutually exclusive** — a body carries one, never both.
- **No `status_code` in the body.** The HTTP status line is authoritative (RFC 9110); duplicating it
  in the body is redundant and can drift.
- `error.code` is a stable `SCREAMING_SNAKE_CASE` identifier (the i18n key); `error.message` is
  **English developer prose for logs/debugging** — clients localize from the **code**, never by
  displaying `message`. 5xx messages are masked to a generic string (the real cause is logged, never
  leaked to the client).
- `meta.pagination` is reserved for when list endpoints adopt pagination; collections currently
  return the full array with no `pagination` field.

### 2. Error codes (granular where the client needs distinct copy)

A single `domain::error_code::ErrorCode` enum is the registry the client mirrors. It is granular for
validation (where distinct localized messages matter) and generic where the HTTP route already
conveys the resource:

`INVALID_EMAIL`, `PASSWORD_TOO_SHORT`, `EMAIL_ALREADY_EXISTS`, `INVALID_LANGUAGE`, `UNAUTHORIZED`,
`INVALID_PLATE_NUMBER`, `ODOMETER_DECREASED`, `INVALID_FUEL_TYPE`, `INVALID_REMINDER_TYPE`,
`MISSING_DUE_DATE`, `MISSING_DUE_ODOMETER`, `INVALID_CATEGORY`, `INVALID_DOC_TYPE`, `NOT_FOUND`,
`CONFLICT`, `VALIDATION`, `INTERNAL`.

The error chain is unchanged in shape (`DomainError → AppError → IntoResponse`); each layer now
carries the code. `DomainError::code()` and `AppError::code()` are exhaustive `match`es, so a new
error variant cannot ship without a code (compile error otherwise).

### 3. request_id propagation

A middleware generates a UUID per request, stores it in a `tokio::task_local!`, and echoes it as the
`X-Request-Id` response header. Both the success envelope and the error envelope read `meta.request_id`
from that task-local — one source of truth, no extractor threaded through every handler, and it works
for error responses (which are built without request context).

### 4. i18n strategy (where localization lives)

- **Client owns all UI copy** (labels, buttons, screens, empty states) and **maps `error.code` →
  localized message**. This is the center of gravity.
- **Backend returns codes, not localized prose**, for synchronous API errors.
- **Backend localizes only server-generated async content** (future reminder notifications, emails)
  using a stored per-user language — because no client is in the loop when a scheduled job renders
  that text.
- A `users.preferred_language` column (`TEXT NOT NULL DEFAULT 'en'`, CHECK `IN ('en','id')`) +
  domain `Language` value object stores it; `PATCH /me { preferred_language }` updates it; `GET /me`
  returns it. This is prep for notifications — no async consumer exists yet, but the column + endpoint
  are the contract.

### 5. Deliberate non-enveloped responses

- `GET /health` stays bare (`{ "status": "ok", "version": "..." }`) — a liveness/ops endpoint, not a
  data resource; monitoring tools expect a minimal shape.
- `204 No Content` (delete, cookie-mode logout) and cookie-only refresh have no body to envelope.

---

## Rationale

- **One parser, one error handler on the client.** The Flutter client switches on HTTP status and
  reads `data` or `error.code` uniformly.
- **Localization single-sourced.** Codes keep the API locale-agnostic and stable; the client is the
  single owner of display strings, so server and client copy cannot drift.
- **Stable contract.** A `code` is a versioned identifier; English `message` text can be reworded
  without breaking clients.
- **`request_id` everywhere** aids support/debugging across the mobile↔backend boundary, on both
  success and error.
- **Portfolio value.** A clean, consistent envelope + code-based i18n reads as production-grade craft.

---

## Consequences

### Positive

- Uniform success/error shape; the client has one response model.
- i18n is correct by construction: codes for sync errors, stored language for async server content.
- New error variants are forced to declare a code (compile-time exhaustiveness).
- `X-Request-Id` + `meta.request_id` give end-to-end correlation.

### Negative / Trade-offs

- **Breaking contract change.** Every success body is now nested under `data`; collections lost their
  named wrapper key. Existing clients (the in-progress Flutter app) must update their parsing — the
  mobile `dio_error_mapper` + DTO parsing is a required follow-up.
- The custom envelope is less "standards-flex" than RFC 9457 problem+json for errors; accepted for
  client-parser uniformity (one shape for success and error).
- `users.preferred_language` has no runtime consumer yet (notifications are future work) — a small,
  deliberate exception to the project's YAGNI rule, justified by the additive-migration cost of
  retrofitting it later and by completing the i18n contract now.

### Migration / Impact

- New modules: `domain/error_code.rs` (`ErrorCode`), `adapters/inbound/http/response.rs`
  (`ApiResponse<T>` + `Meta`), `adapters/inbound/http/request_id.rs` (task-local + middleware),
  `application/user/update_language.rs`.
- `AppError::Conflict`/`Validation` now carry `{ code, message }`; constructors `AppError::validation`
  / `AppError::conflict`.
- All handlers return `ApiResponse::ok` / `ApiResponse::created`; the list-wrapper DTOs
  (`VehicleListResponse`, etc.) were removed (collections return the bare array as `data`).
- Migration `20260628000001_user_preferred_language.sql`; `User` gains `preferred_language: Language`;
  `UserRepository` gains `update_language`; route `PATCH /me`.
- **Follow-up (not in this change):** update the mobile client's error mapper + response parsing to
  the envelope; update the OpenAPI spec (`packages/openapi/veyra.yaml`) to document the envelope and
  the `ErrorCode` set; add `meta.pagination` when list pagination lands.
