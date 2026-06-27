# Veyra Mobile — Plan 1: Scaffold + Core + Auth

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> **Presentation tasks (screens/widgets):** invoke `frontend-design` first (+ `ui-ux-pro-max`) — apply the Veyra brand (dark-first, accent `#F26A21`, logo). **Domain/data tasks:** plain Dart discipline + the Dart quality gate below. Honor each task's `TDD:` verdict.

**Goal:** Stand up the Flutter app skeleton, the cross-cutting `core/` layer (theme, networking, secure-storage tokens, routing, error model), and a complete `auth` feature (login / register / logout / me) built in the hexagonal + DDD-tactical shape that every later feature replicates.

**Architecture:** Hexagonal per feature — `presentation → domain ← data`, domain is pure Dart (no `flutter`/`dio`/`dart:io`). DDD tactical blocks: value objects (validated), entities, repository ports, use cases. Riverpod is DI + state. dio + interceptors handle bearer auth, single-flight refresh, and error→`Failure` mapping.

**Tech Stack:** Flutter 3.x · Dart 3.x · flutter_riverpod · dio · go_router · fpdart (`Either`/`Unit`) · freezed + json_serializable · flutter_secure_storage · decimal · mocktail · very_good_analysis

**Spec:** `docs/superpowers/specs/2026-06-27-veyra-mobile-app-design.md` · **Backend dep:** ADR-0007 bearer mode (`docs/superpowers/plans/2026-06-27-mobile-bearer-auth.md`)

## Global Constraints

- Dart 3 null-safety; no `dynamic`; no `print()` (use a logger or `debugPrint` only in dev paths).
- **Domain layer imports nothing from `package:flutter`, `package:dio`, or `dart:io`** — enforced by `tool/check_boundaries.dart`.
- Value objects validate in a static `create()` returning `Either<ValidationFailure, T>`; the unchecked constructor is private — an invalid instance is unrepresentable.
- Use cases return `Future<Either<Failure, T>>`. Presentation unwraps `Either`; it never sees a raw `DioException`.
- Currency uses `Decimal` (package `decimal`), never `double`.
- Bearer auth: attach `Authorization: Bearer <access>` to protected requests and `X-Auth-Mode: bearer` to the four auth endpoints. Tokens live in `flutter_secure_storage`. Refresh **rotates** — persist the new refresh token on every successful refresh.
- API error body is the flat shape `{"error":"<message>"}` — parse `response.data['error']` as a String. Do NOT expect a nested `{"error":{"code","message"}}`.
- DRY: one `mapDioError` (DioException→Failure), one `TokenStore`, one `wants_bearer`-equivalent (the AuthInterceptor owns the header logic).
- Verify before "done": `dart format --set-exit-if-changed . && flutter analyze && flutter test && dart run tool/check_boundaries.dart`.

### Dart quality gate

- `flutter analyze` clean under `very_good_analysis`.
- Tests: domain value objects + use cases (TDD), data mappers, key widgets.
- No business logic in widgets — they watch a controller/provider and render.
- Private VO constructors; `create()` is the only public path.

---

## File Structure (this plan)

```
apps/mobile/
├── pubspec.yaml
├── analysis_options.yaml
├── tool/check_boundaries.dart
├── lib/
│   ├── main.dart
│   ├── core/
│   │   ├── config/app_config.dart
│   │   ├── error/failure.dart
│   │   ├── error/dio_error_mapper.dart
│   │   ├── storage/token_store.dart
│   │   ├── network/dio_client.dart
│   │   ├── network/auth_interceptor.dart
│   │   ├── network/refresh_interceptor.dart
│   │   ├── theme/app_theme.dart
│   │   └── router/app_router.dart
│   └── features/auth/
│       ├── domain/
│       │   ├── entities/user.dart
│       │   ├── value_objects/email.dart
│       │   ├── value_objects/password.dart
│       │   ├── repositories/auth_repository.dart
│       │   └── usecases/{login,register,logout,get_me}_usecase.dart
│       ├── data/
│       │   ├── models/{user_dto,auth_tokens_dto}.dart
│       │   ├── datasources/auth_remote_data_source.dart
│       │   └── repositories/auth_repository_impl.dart
│       └── presentation/
│           ├── controllers/auth_controller.dart
│           └── screens/{login,register}_screen.dart
└── test/
    ├── features/auth/domain/{email,password,login_usecase}_test.dart
    └── features/auth/data/user_dto_test.dart
```

---

### Task 1: Project scaffold + tooling

**TDD: no** — project setup; verified by `flutter analyze` + the boundary tool running clean.

**Files:** create `apps/mobile/` via `flutter create`, then `pubspec.yaml`, `analysis_options.yaml`, `tool/check_boundaries.dart`.

- [ ] **Step 1: Create the Flutter project**

```bash
cd apps/mobile  # if missing: cd apps && flutter create --org dev.oksasatya --project-name veyra_mobile mobile && cd mobile
```

- [ ] **Step 2: Set `pubspec.yaml` dependencies**

```yaml
name: veyra_mobile
description: Veyra vehicle-management mobile client.
publish_to: "none"
environment:
  sdk: ">=3.4.0 <4.0.0"

dependencies:
  flutter:
    sdk: flutter
  flutter_riverpod: ^2.5.1
  dio: ^5.7.0
  go_router: ^14.2.0
  fpdart: ^1.1.0
  freezed_annotation: ^2.4.4
  json_annotation: ^4.9.0
  flutter_secure_storage: ^9.2.2
  decimal: ^3.0.2

dev_dependencies:
  flutter_test:
    sdk: flutter
  build_runner: ^2.4.11
  freezed: ^2.5.7
  json_serializable: ^6.8.0
  mocktail: ^1.0.4
  very_good_analysis: ^6.0.0

flutter:
  uses-material-design: true
```

- [ ] **Step 3: Set `analysis_options.yaml`**

```yaml
include: package:very_good_analysis/analysis_options.yaml
analyzer:
  exclude:
    - "**/*.g.dart"
    - "**/*.freezed.dart"
linter:
  rules:
    public_member_api_docs: false
```

- [ ] **Step 4: Create the domain-boundary check `tool/check_boundaries.dart`**

```dart
// Fails (exit 1) if any file under a `domain/` directory imports Flutter, dio,
// or dart:io — the client analogue of the backend's ci/check-boundaries.sh.
import 'dart:io';

void main() {
  final forbidden = [
    RegExp(r'''import\s+['"]package:flutter/'''),
    RegExp(r'''import\s+['"]package:dio/'''),
    RegExp(r'''import\s+['"]dart:io'''),
  ];
  final violations = <String>[];
  final lib = Directory('lib');
  for (final entity in lib.listSync(recursive: true)) {
    if (entity is! File || !entity.path.endsWith('.dart')) continue;
    if (!entity.path.contains('${Platform.pathSeparator}domain${Platform.pathSeparator}')) {
      continue;
    }
    final src = entity.readAsStringSync();
    for (final re in forbidden) {
      if (re.hasMatch(src)) {
        violations.add('${entity.path}: forbidden import (${re.pattern})');
      }
    }
  }
  if (violations.isNotEmpty) {
    stderr.writeln('Domain boundary violations:');
    for (final v in violations) {
      stderr.writeln('  $v');
    }
    exit(1);
  }
  stdout.writeln('domain boundary OK');
}
```

- [ ] **Step 5: Install + verify**

```bash
flutter pub get
flutter analyze
dart run tool/check_boundaries.dart   # expected: "domain boundary OK"
```

- [ ] **Step 6: Commit**

```bash
git add apps/mobile/pubspec.yaml apps/mobile/analysis_options.yaml apps/mobile/tool/ apps/mobile/lib apps/mobile/ios apps/mobile/android
git commit -m "feat(mobile): scaffold Flutter app + deps + domain-boundary check"
```

---

### Task 2: Core error model

**TDD: no** — sealed data types; exercised by later use-case tests.

**Files:** create `lib/core/error/failure.dart`, `lib/core/error/dio_error_mapper.dart`.

**Interfaces:**
- Produces: `sealed class Failure` + variants; `Failure mapDioError(DioException e)`.

- [ ] **Step 1: Create `lib/core/error/failure.dart`**

```dart
/// Application-facing failures. Use cases return `Either<Failure, T>`; the UI
/// renders these, never a raw transport error.
sealed class Failure {
  const Failure(this.message);
  final String message;
}

class NetworkFailure extends Failure {
  const NetworkFailure([super.message = 'no connection']);
}

class ServerFailure extends Failure {
  const ServerFailure([super.message = 'server error']);
}

class UnauthorizedFailure extends Failure {
  const UnauthorizedFailure([super.message = 'unauthorized']);
}

class NotFoundFailure extends Failure {
  const NotFoundFailure([super.message = 'not found']);
}

class ConflictFailure extends Failure {
  const ConflictFailure([super.message = 'conflict']);
}

class ValidationFailure extends Failure {
  const ValidationFailure({required String message, this.field}) : super(message);
  final String? field;
}
```

- [ ] **Step 2: Create `lib/core/error/dio_error_mapper.dart`**

```dart
import 'package:dio/dio.dart';

import 'failure.dart';

/// Map a [DioException] to a [Failure], reading the backend's flat error body
/// `{"error":"<message>"}` for a human message when present.
Failure mapDioError(DioException e) {
  final status = e.response?.statusCode;
  final message = _extractMessage(e.response?.data);
  return switch (status) {
    401 => UnauthorizedFailure(message ?? 'unauthorized'),
    404 => NotFoundFailure(message ?? 'not found'),
    409 => ConflictFailure(message ?? 'conflict'),
    422 => ValidationFailure(message: message ?? 'validation failed'),
    >= 500 => ServerFailure(message ?? 'server error'),
    null => const NetworkFailure(),
    _ => ServerFailure(message ?? 'unexpected error'),
  };
}

String? _extractMessage(Object? data) {
  if (data is Map && data['error'] is String) {
    return data['error'] as String;
  }
  return null;
}
```

- [ ] **Step 3: Verify + commit**

```bash
flutter analyze
git add apps/mobile/lib/core/error/
git commit -m "feat(mobile): sealed Failure hierarchy + DioException mapper"
```

---

### Task 3: Token store + app config

**TDD: no** — thin wrapper over `flutter_secure_storage`; behavior covered via the interceptor test in Task 5.

**Files:** create `lib/core/config/app_config.dart`, `lib/core/storage/token_store.dart`.

**Interfaces:**
- Produces: `appConfigProvider` (base URL); `TokenStore` (`read/saveTokens/clear`); `tokenStoreProvider`.

- [ ] **Step 1: Create `lib/core/config/app_config.dart`**

```dart
import 'package:flutter_riverpod/flutter_riverpod.dart';

class AppConfig {
  const AppConfig({required this.apiBaseUrl});
  // Override at build: --dart-define=API_BASE_URL=https://api.veyra.dev
  final String apiBaseUrl;

  factory AppConfig.fromEnv() => const AppConfig(
        apiBaseUrl: String.fromEnvironment(
          'API_BASE_URL',
          defaultValue: 'http://localhost:3000',
        ),
      );
}

final appConfigProvider = Provider<AppConfig>((ref) => AppConfig.fromEnv());
```

- [ ] **Step 2: Create `lib/core/storage/token_store.dart`**

```dart
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';

/// Persisted bearer tokens. `refresh` is the opaque `{family_id}.{raw_secret}`
/// string — stored and replayed verbatim, never parsed on the client.
class Tokens {
  const Tokens({required this.access, required this.refresh});
  final String access;
  final String refresh;
}

class TokenStore {
  TokenStore(this._storage);
  final FlutterSecureStorage _storage;

  static const _kAccess = 'veyra_access';
  static const _kRefresh = 'veyra_refresh';

  Future<Tokens?> read() async {
    final access = await _storage.read(key: _kAccess);
    final refresh = await _storage.read(key: _kRefresh);
    if (access == null || refresh == null) return null;
    return Tokens(access: access, refresh: refresh);
  }

  Future<void> save(Tokens tokens) async {
    await _storage.write(key: _kAccess, value: tokens.access);
    await _storage.write(key: _kRefresh, value: tokens.refresh);
  }

  Future<void> clear() async {
    await _storage.delete(key: _kAccess);
    await _storage.delete(key: _kRefresh);
  }
}

final tokenStoreProvider = Provider<TokenStore>(
  (ref) => TokenStore(const FlutterSecureStorage()),
);
```

- [ ] **Step 3: Verify + commit**

```bash
flutter analyze
git add apps/mobile/lib/core/config/ apps/mobile/lib/core/storage/
git commit -m "feat(mobile): AppConfig (base URL) + secure-storage TokenStore"
```

---

### Task 4: Auth domain — value objects (TDD)

**TDD: yes** — `Email`/`Password` validation are pure functions with clear accept/reject contracts.

**Files:** create `lib/features/auth/domain/value_objects/{email,password}.dart`; tests `test/features/auth/domain/{email,password}_test.dart`.

**Interfaces:**
- Produces: `Email.create(String) -> Either<ValidationFailure, Email>` (+ `.value`); `Password.create(String) -> Either<ValidationFailure, Password>`.

- [ ] **Step 1: Write failing tests `test/features/auth/domain/email_test.dart`**

```dart
import 'package:flutter_test/flutter_test.dart';
import 'package:fpdart/fpdart.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/features/auth/domain/value_objects/email.dart';

void main() {
  test('valid email accepted + lowercased', () {
    final r = Email.create('Alice@Example.COM');
    expect(r.isRight(), true);
    r.match((_) => fail('expected right'), (e) => expect(e.value, 'alice@example.com'));
  });

  test('email without @ rejected', () {
    expect(Email.create('nope').isLeft(), true);
  });

  test('empty email rejected', () {
    expect(Email.create('').isLeft(), true);
  });

  test('no domain dot rejected', () {
    expect(Email.create('a@nodot').isLeft(), true);
  });
}
```

`test/features/auth/domain/password_test.dart`:

```dart
import 'package:flutter_test/flutter_test.dart';
import 'package:veyra_mobile/features/auth/domain/value_objects/password.dart';

void main() {
  test('password >= 8 chars accepted', () {
    expect(Password.create('password123').isRight(), true);
  });

  test('password < 8 chars rejected', () {
    expect(Password.create('short').isLeft(), true);
  });
}
```

- [ ] **Step 2: Run to verify failure**

```bash
cd apps/mobile && flutter test test/features/auth/domain/email_test.dart
# Expected: compile error — Email not defined.
```

- [ ] **Step 3: Implement `lib/features/auth/domain/value_objects/email.dart`**

```dart
import 'package:fpdart/fpdart.dart';

import '../../../../core/error/failure.dart';

/// A validated, normalized (trimmed + lowercased) email address.
class Email {
  const Email._(this.value);
  final String value;

  static Either<ValidationFailure, Email> create(String raw) {
    final v = raw.trim().toLowerCase();
    if (v.isEmpty || !v.contains('@')) {
      return const Left(ValidationFailure(message: 'invalid email', field: 'email'));
    }
    final parts = v.split('@');
    if (parts.length != 2 || parts.first.isEmpty || !parts.last.contains('.')) {
      return const Left(ValidationFailure(message: 'invalid email', field: 'email'));
    }
    return Right(Email._(v));
  }
}
```

- [ ] **Step 4: Implement `lib/features/auth/domain/value_objects/password.dart`**

```dart
import 'package:fpdart/fpdart.dart';

import '../../../../core/error/failure.dart';

/// A password that satisfies the minimum-length policy (8 chars).
class Password {
  const Password._(this.value);
  final String value;

  static Either<ValidationFailure, Password> create(String raw) {
    if (raw.length < 8) {
      return const Left(
        ValidationFailure(message: 'password too short — minimum 8 characters', field: 'password'),
      );
    }
    return Right(Password._(raw));
  }
}
```

- [ ] **Step 5: Run tests + commit**

```bash
flutter test test/features/auth/domain/
# Expected: all pass
git add apps/mobile/lib/features/auth/domain/value_objects/ apps/mobile/test/features/auth/domain/
git commit -m "feat(mobile): auth Email + Password value objects (validated)"
```

---

### Task 5: Auth domain — entity, port, use cases (TDD) + dio interceptors

**TDD: yes** for use cases; **no** for the interceptors (covered by an interceptor test at the end).

**Files:**
- domain: `entities/user.dart`, `repositories/auth_repository.dart`, `usecases/{login,register,logout,get_me}_usecase.dart`
- core: `network/dio_client.dart`, `network/auth_interceptor.dart`, `network/refresh_interceptor.dart`
- tests: `test/features/auth/domain/login_usecase_test.dart`

**Interfaces:**
- Produces: `User`; `AuthRepository` port; `LoginUseCase`/`RegisterUseCase`/`LogoutUseCase`/`GetMeUseCase`; `dioProvider`.

- [ ] **Step 1: Create `lib/features/auth/domain/entities/user.dart`**

```dart
/// Domain entity — pure Dart, no JSON. The data layer maps a DTO into this.
class User {
  const User({required this.id, required this.email, required this.name});
  final String id;
  final String email;
  final String name;
}
```

- [ ] **Step 2: Create the port `lib/features/auth/domain/repositories/auth_repository.dart`**

```dart
import 'package:fpdart/fpdart.dart';

import '../../../../core/error/failure.dart';
import '../entities/user.dart';
import '../value_objects/email.dart';
import '../value_objects/password.dart';

abstract interface class AuthRepository {
  Future<Either<Failure, User>> login({required Email email, required Password password});
  Future<Either<Failure, User>> register({
    required Email email,
    required Password password,
    required String name,
  });
  Future<Either<Failure, Unit>> logout();
  Future<Either<Failure, User>> getMe();
}
```

- [ ] **Step 3: Create the use cases**

`usecases/login_usecase.dart`:

```dart
import 'package:fpdart/fpdart.dart';

import '../../../../core/error/failure.dart';
import '../entities/user.dart';
import '../repositories/auth_repository.dart';
import '../value_objects/email.dart';
import '../value_objects/password.dart';

class LoginUseCase {
  const LoginUseCase(this._repo);
  final AuthRepository _repo;

  Future<Either<Failure, User>> call(Email email, Password password) =>
      _repo.login(email: email, password: password);
}
```

`register_usecase.dart` (full — do not abbreviate):

```dart
import 'package:fpdart/fpdart.dart';

import '../../../../core/error/failure.dart';
import '../entities/user.dart';
import '../repositories/auth_repository.dart';
import '../value_objects/email.dart';
import '../value_objects/password.dart';

class RegisterUseCase {
  const RegisterUseCase(this._repo);
  final AuthRepository _repo;

  Future<Either<Failure, User>> call(Email email, Password password, String name) =>
      _repo.register(email: email, password: password, name: name);
}
```

`logout_usecase.dart`:

```dart
import 'package:fpdart/fpdart.dart';

import '../../../../core/error/failure.dart';
import '../repositories/auth_repository.dart';

class LogoutUseCase {
  const LogoutUseCase(this._repo);
  final AuthRepository _repo;

  Future<Either<Failure, Unit>> call() => _repo.logout();
}
```

`get_me_usecase.dart`:

```dart
import 'package:fpdart/fpdart.dart';

import '../../../../core/error/failure.dart';
import '../entities/user.dart';
import '../repositories/auth_repository.dart';

class GetMeUseCase {
  const GetMeUseCase(this._repo);
  final AuthRepository _repo;

  Future<Either<Failure, User>> call() => _repo.getMe();
}
```

- [ ] **Step 4: Write failing use-case test `test/features/auth/domain/login_usecase_test.dart`**

```dart
import 'package:flutter_test/flutter_test.dart';
import 'package:fpdart/fpdart.dart';
import 'package:mocktail/mocktail.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/features/auth/domain/entities/user.dart';
import 'package:veyra_mobile/features/auth/domain/repositories/auth_repository.dart';
import 'package:veyra_mobile/features/auth/domain/usecases/login_usecase.dart';
import 'package:veyra_mobile/features/auth/domain/value_objects/email.dart';
import 'package:veyra_mobile/features/auth/domain/value_objects/password.dart';

class _MockRepo extends Mock implements AuthRepository {}

void main() {
  late _MockRepo repo;
  late LoginUseCase uc;
  final email = Email.create('a@b.com').getRight().toNullable()!;
  final password = Password.create('password123').getRight().toNullable()!;

  setUp(() {
    repo = _MockRepo();
    uc = LoginUseCase(repo);
  });

  test('delegates to repo and returns the user', () async {
    const user = User(id: '1', email: 'a@b.com', name: 'A');
    when(() => repo.login(email: email, password: password))
        .thenAnswer((_) async => const Right(user));

    final result = await uc(email, password);

    expect(result.getRight().toNullable(), user);
  });

  test('propagates a failure', () async {
    when(() => repo.login(email: email, password: password))
        .thenAnswer((_) async => const Left(UnauthorizedFailure()));

    final result = await uc(email, password);

    expect(result.isLeft(), true);
  });
}
```

- [ ] **Step 5: Run it (passes — use case is a thin delegate)**

```bash
flutter test test/features/auth/domain/login_usecase_test.dart
# Expected: pass
```

- [ ] **Step 6: Create the dio client `lib/core/network/dio_client.dart`**

```dart
import 'package:dio/dio.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../config/app_config.dart';
import '../storage/token_store.dart';
import 'auth_interceptor.dart';
import 'refresh_interceptor.dart';

final dioProvider = Provider<Dio>((ref) {
  final config = ref.watch(appConfigProvider);
  final store = ref.watch(tokenStoreProvider);
  final dio = Dio(BaseOptions(
    baseUrl: config.apiBaseUrl,
    connectTimeout: const Duration(seconds: 10),
    receiveTimeout: const Duration(seconds: 10),
  ));
  dio.interceptors.add(AuthInterceptor(store));
  dio.interceptors.add(RefreshInterceptor(dio: dio, store: store, onSessionExpired: () {
    ref.read(sessionExpiredProvider.notifier).state = true;
  }));
  return dio;
});

/// Flips true when refresh fails — the router redirects to /login.
final sessionExpiredProvider = StateProvider<bool>((ref) => false);
```

- [ ] **Step 7: Create `lib/core/network/auth_interceptor.dart`**

```dart
import 'package:dio/dio.dart';

import '../storage/token_store.dart';

const _authPaths = {'/auth/register', '/auth/login', '/auth/refresh', '/auth/logout'};

/// Attaches `Authorization: Bearer <access>` to protected requests and
/// `X-Auth-Mode: bearer` to the four auth endpoints.
class AuthInterceptor extends Interceptor {
  AuthInterceptor(this._store);
  final TokenStore _store;

  @override
  Future<void> onRequest(RequestOptions options, RequestInterceptorHandler handler) async {
    if (_authPaths.contains(options.path)) {
      options.headers['X-Auth-Mode'] = 'bearer';
    }
    final tokens = await _store.read();
    if (tokens != null && !_authPaths.contains(options.path)) {
      options.headers['Authorization'] = 'Bearer ${tokens.access}';
    }
    handler.next(options);
  }
}
```

- [ ] **Step 8: Create `lib/core/network/refresh_interceptor.dart` (single-flight)**

```dart
import 'dart:async';

import 'package:dio/dio.dart';

import '../storage/token_store.dart';

/// On 401, performs ONE `POST /auth/refresh` even under concurrent failures
/// (single-flight via a shared Future), persists the rotated tokens, and retries
/// the original request. If refresh fails, clears tokens and signals expiry.
class RefreshInterceptor extends Interceptor {
  RefreshInterceptor({required this.dio, required this.store, required this.onSessionExpired});
  final Dio dio;
  final TokenStore store;
  final void Function() onSessionExpired;

  Future<bool>? _inFlight;

  @override
  Future<void> onError(DioException err, ErrorInterceptorHandler handler) async {
    final is401 = err.response?.statusCode == 401;
    final isRefreshCall = err.requestOptions.path == '/auth/refresh';
    if (!is401 || isRefreshCall) {
      return handler.next(err);
    }

    final ok = await (_inFlight ??= _refresh());
    _inFlight = null;
    if (!ok) {
      onSessionExpired();
      return handler.next(err);
    }

    try {
      final tokens = await store.read();
      final req = err.requestOptions;
      req.headers['Authorization'] = 'Bearer ${tokens!.access}';
      final response = await dio.fetch<dynamic>(req);
      return handler.resolve(response);
    } on DioException catch (e) {
      return handler.next(e);
    }
  }

  Future<bool> _refresh() async {
    final tokens = await store.read();
    if (tokens == null) return false;
    try {
      final res = await dio.post<Map<String, dynamic>>(
        '/auth/refresh',
        data: {'refresh_token': tokens.refresh},
        options: Options(headers: {'X-Auth-Mode': 'bearer'}),
      );
      final t = res.data!['tokens'] as Map<String, dynamic>;
      await store.save(Tokens(
        access: t['access_token'] as String,
        refresh: t['refresh_token'] as String,
      ));
      return true;
    } on DioException {
      await store.clear();
      return false;
    }
  }
}
```

- [ ] **Step 9: Verify + commit**

```bash
flutter analyze && flutter test && dart run tool/check_boundaries.dart
git add apps/mobile/lib/features/auth/domain/ apps/mobile/lib/core/network/ apps/mobile/test/features/auth/domain/login_usecase_test.dart
git commit -m "feat(mobile): auth domain (entity/port/use cases) + dio bearer + single-flight refresh"
```

---

### Task 6: Auth data layer (DTOs, datasource, repo impl)

**TDD: yes** for the DTO mapper; the repo impl is covered indirectly + a happy-path test.

**Files:**
- `lib/features/auth/data/models/{user_dto,auth_tokens_dto}.dart`
- `lib/features/auth/data/datasources/auth_remote_data_source.dart`
- `lib/features/auth/data/repositories/auth_repository_impl.dart`
- test `test/features/auth/data/user_dto_test.dart`

**Interfaces:**
- Consumes: `User`, `AuthRepository`, `Tokens`, `TokenStore`, `mapDioError`, `dioProvider`.
- Produces: `authRepositoryProvider` (bound to `AuthRepositoryImpl`), use-case providers.

- [ ] **Step 1: Write failing DTO mapper test `test/features/auth/data/user_dto_test.dart`**

```dart
import 'package:flutter_test/flutter_test.dart';
import 'package:veyra_mobile/features/auth/data/models/user_dto.dart';

void main() {
  test('UserDto.fromJson maps to entity', () {
    final dto = UserDto.fromJson(const {'id': '1', 'email': 'a@b.com', 'name': 'A'});
    final user = dto.toDomain();
    expect(user.id, '1');
    expect(user.email, 'a@b.com');
    expect(user.name, 'A');
  });
}
```

- [ ] **Step 2: Create `lib/features/auth/data/models/user_dto.dart`**

```dart
import 'package:json_annotation/json_annotation.dart';

import '../../domain/entities/user.dart';

part 'user_dto.g.dart';

@JsonSerializable()
class UserDto {
  const UserDto({required this.id, required this.email, required this.name});
  factory UserDto.fromJson(Map<String, dynamic> json) => _$UserDtoFromJson(json);
  final String id;
  final String email;
  final String name;

  User toDomain() => User(id: id, email: email, name: name);
}
```

- [ ] **Step 3: Create `lib/features/auth/data/models/auth_tokens_dto.dart`**

```dart
import 'package:json_annotation/json_annotation.dart';

import '../../../../core/storage/token_store.dart';

part 'auth_tokens_dto.g.dart';

@JsonSerializable()
class AuthTokensDto {
  const AuthTokensDto({required this.accessToken, required this.refreshToken});
  factory AuthTokensDto.fromJson(Map<String, dynamic> json) => _$AuthTokensDtoFromJson(json);
  @JsonKey(name: 'access_token')
  final String accessToken;
  @JsonKey(name: 'refresh_token')
  final String refreshToken;

  Tokens toStore() => Tokens(access: accessToken, refresh: refreshToken);
}
```

- [ ] **Step 4: Run build_runner (generates `*.g.dart`)**

```bash
dart run build_runner build --delete-conflicting-outputs
flutter test test/features/auth/data/user_dto_test.dart   # expected: pass
```

- [ ] **Step 5: Create the remote datasource `lib/features/auth/data/datasources/auth_remote_data_source.dart`**

```dart
import 'package:dio/dio.dart';

import '../models/auth_tokens_dto.dart';
import '../models/user_dto.dart';

/// Raw HTTP calls. Throws DioException on failure (mapped to Failure in the repo).
class AuthRemoteDataSource {
  AuthRemoteDataSource(this._dio);
  final Dio _dio;

  Future<({UserDto user, AuthTokensDto tokens})> login(String email, String password) async {
    final res = await _dio.post<Map<String, dynamic>>(
      '/auth/login',
      data: {'email': email, 'password': password},
    );
    return _parseAuth(res.data!);
  }

  Future<({UserDto user, AuthTokensDto tokens})> register(
      String email, String password, String name) async {
    final res = await _dio.post<Map<String, dynamic>>(
      '/auth/register',
      data: {'email': email, 'password': password, 'name': name},
    );
    return _parseAuth(res.data!);
  }

  Future<void> logout(String refreshToken) =>
      _dio.post<void>('/auth/logout', data: {'refresh_token': refreshToken});

  Future<UserDto> me() async {
    final res = await _dio.get<Map<String, dynamic>>('/me');
    return UserDto.fromJson(res.data!);
  }

  ({UserDto user, AuthTokensDto tokens}) _parseAuth(Map<String, dynamic> data) => (
        user: UserDto.fromJson(data['user'] as Map<String, dynamic>),
        tokens: AuthTokensDto.fromJson(data['tokens'] as Map<String, dynamic>),
      );
}
```

- [ ] **Step 6: Create the repo impl `lib/features/auth/data/repositories/auth_repository_impl.dart`**

```dart
import 'package:dio/dio.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:fpdart/fpdart.dart';

import '../../../../core/error/dio_error_mapper.dart';
import '../../../../core/error/failure.dart';
import '../../../../core/network/dio_client.dart';
import '../../../../core/storage/token_store.dart';
import '../../domain/entities/user.dart';
import '../../domain/repositories/auth_repository.dart';
import '../../domain/usecases/get_me_usecase.dart';
import '../../domain/usecases/login_usecase.dart';
import '../../domain/usecases/logout_usecase.dart';
import '../../domain/usecases/register_usecase.dart';
import '../../domain/value_objects/email.dart';
import '../../domain/value_objects/password.dart';
import '../datasources/auth_remote_data_source.dart';

class AuthRepositoryImpl implements AuthRepository {
  AuthRepositoryImpl({required this.remote, required this.store});
  final AuthRemoteDataSource remote;
  final TokenStore store;

  @override
  Future<Either<Failure, User>> login({required Email email, required Password password}) =>
      _authCall(() => remote.login(email.value, password.value));

  @override
  Future<Either<Failure, User>> register({
    required Email email,
    required Password password,
    required String name,
  }) =>
      _authCall(() => remote.register(email.value, password.value, name));

  @override
  Future<Either<Failure, Unit>> logout() async {
    try {
      final tokens = await store.read();
      if (tokens != null) await remote.logout(tokens.refresh);
      await store.clear();
      return const Right(unit);
    } on DioException catch (e) {
      await store.clear(); // best-effort local logout regardless
      return Left(mapDioError(e));
    }
  }

  @override
  Future<Either<Failure, User>> getMe() async {
    try {
      final dto = await remote.me();
      return Right(dto.toDomain());
    } on DioException catch (e) {
      return Left(mapDioError(e));
    }
  }

  Future<Either<Failure, User>> _authCall(
      Future<({UserDto user, AuthTokensDto tokens})> Function() call) async {
    try {
      final result = await call();
      await store.save(result.tokens.toStore());
      return Right(result.user.toDomain());
    } on DioException catch (e) {
      return Left(mapDioError(e));
    }
  }
}

// ── Providers (DI) ─────────────────────────────────────────────────────────────

final authRepositoryProvider = Provider<AuthRepository>((ref) => AuthRepositoryImpl(
      remote: AuthRemoteDataSource(ref.watch(dioProvider)),
      store: ref.watch(tokenStoreProvider),
    ));

final loginUseCaseProvider = Provider((ref) => LoginUseCase(ref.watch(authRepositoryProvider)));
final registerUseCaseProvider = Provider((ref) => RegisterUseCase(ref.watch(authRepositoryProvider)));
final logoutUseCaseProvider = Provider((ref) => LogoutUseCase(ref.watch(authRepositoryProvider)));
final getMeUseCaseProvider = Provider((ref) => GetMeUseCase(ref.watch(authRepositoryProvider)));
```

> Note the DTO imports (`UserDto`, `AuthTokensDto`) needed for the `_authCall` signature — add them.

- [ ] **Step 7: Verify + commit**

```bash
flutter analyze && flutter test && dart run tool/check_boundaries.dart
git add apps/mobile/lib/features/auth/data/ apps/mobile/test/features/auth/data/
git commit -m "feat(mobile): auth data layer (DTOs, datasource, repository impl + DI)"
```

---

### Task 7: Auth presentation — controller + screens

**TDD: no** for screens (verify by running + a widget test); the controller logic is simple delegation.

**Files:**
- `lib/features/auth/presentation/controllers/auth_controller.dart`
- `lib/features/auth/presentation/screens/{login,register}_screen.dart`
- test `test/features/auth/presentation/login_screen_test.dart`

**Interfaces:**
- Consumes: use-case providers, `Email`/`Password`, `TokenStore`.
- Produces: `authControllerProvider` (`AsyncNotifier`-style auth state), `AuthState`.

**Invoke `frontend-design` + `ui-ux-pro-max` first** — apply the brand (dark, accent `#F26A21`, logo on the auth screens).

- [ ] **Step 1: Create the controller `lib/features/auth/presentation/controllers/auth_controller.dart`**

```dart
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../domain/entities/user.dart';
import '../../domain/usecases/get_me_usecase.dart';
import '../../domain/usecases/login_usecase.dart';
import '../../domain/usecases/logout_usecase.dart';
import '../../domain/usecases/register_usecase.dart';
import '../../domain/value_objects/email.dart';
import '../../domain/value_objects/password.dart';
import '../../../../core/error/failure.dart';
import '../../../../core/storage/token_store.dart';

sealed class AuthState {
  const AuthState();
}

class AuthLoading extends AuthState {
  const AuthLoading();
}

class Authenticated extends AuthState {
  const Authenticated(this.user);
  final User user;
}

class Unauthenticated extends AuthState {
  const Unauthenticated([this.failure]);
  final Failure? failure;
}

class AuthController extends AsyncNotifier<AuthState> {
  @override
  Future<AuthState> build() async {
    final tokens = await ref.read(tokenStoreProvider).read();
    if (tokens == null) return const Unauthenticated();
    final result = await ref.read(getMeUseCaseProvider)();
    return result.match((f) => const Unauthenticated(), Authenticated.new);
  }

  Future<void> login(Email email, Password password) async {
    state = const AsyncData(AuthLoading());
    final result = await ref.read(loginUseCaseProvider)(email, password);
    state = AsyncData(result.match(Unauthenticated.new, Authenticated.new));
  }

  Future<void> register(Email email, Password password, String name) async {
    state = const AsyncData(AuthLoading());
    final result = await ref.read(registerUseCaseProvider)(email, password, name);
    state = AsyncData(result.match(Unauthenticated.new, Authenticated.new));
  }

  Future<void> logout() async {
    await ref.read(logoutUseCaseProvider)();
    state = const AsyncData(Unauthenticated());
  }
}

final authControllerProvider =
    AsyncNotifierProvider<AuthController, AuthState>(AuthController.new);
```

- [ ] **Step 2: Create `login_screen.dart`** (brand-styled; validates VOs before calling the controller)

```dart
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../domain/value_objects/email.dart';
import '../../domain/value_objects/password.dart';
import '../controllers/auth_controller.dart';

class LoginScreen extends ConsumerStatefulWidget {
  const LoginScreen({super.key});
  @override
  ConsumerState<LoginScreen> createState() => _LoginScreenState();
}

class _LoginScreenState extends ConsumerState<LoginScreen> {
  final _email = TextEditingController();
  final _password = TextEditingController();
  String? _error;

  @override
  void dispose() {
    _email.dispose();
    _password.dispose();
    super.dispose();
  }

  void _submit() {
    final email = Email.create(_email.text);
    final password = Password.create(_password.text);
    email.match(
      (f) => setState(() => _error = f.message),
      (e) => password.match(
        (f) => setState(() => _error = f.message),
        (p) {
          setState(() => _error = null);
          ref.read(authControllerProvider.notifier).login(e, p);
        },
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(authControllerProvider);
    return Scaffold(
      body: Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text('veyra', style: Theme.of(context).textTheme.headlineLarge),
              const SizedBox(height: 24),
              TextField(controller: _email, decoration: const InputDecoration(labelText: 'Email')),
              const SizedBox(height: 12),
              TextField(
                controller: _password,
                obscureText: true,
                decoration: const InputDecoration(labelText: 'Password'),
              ),
              if (_error != null) ...[
                const SizedBox(height: 12),
                Text(_error!, style: TextStyle(color: Theme.of(context).colorScheme.error)),
              ],
              const SizedBox(height: 24),
              FilledButton(
                onPressed: state.isLoading ? null : _submit,
                child: const Text('Log in'),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
```

- [ ] **Step 3: Create `register_screen.dart`** (same shape with a name field — write it out fully, mirroring login + a `TextField` for `name` and calling `ref.read(authControllerProvider.notifier).register(...)`).

- [ ] **Step 4: Write a widget test `test/features/auth/presentation/login_screen_test.dart`**

```dart
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:veyra_mobile/features/auth/presentation/screens/login_screen.dart';

void main() {
  testWidgets('shows validation error for bad email', (tester) async {
    await tester.pumpWidget(const ProviderScope(child: MaterialApp(home: LoginScreen())));
    await tester.enterText(find.byType(TextField).first, 'notanemail');
    await tester.enterText(find.byType(TextField).last, 'password123');
    await tester.tap(find.text('Log in'));
    await tester.pump();
    expect(find.text('invalid email'), findsOneWidget);
  });
}
```

- [ ] **Step 5: Run + commit**

```bash
flutter analyze && flutter test
git add apps/mobile/lib/features/auth/presentation/ apps/mobile/test/features/auth/presentation/
git commit -m "feat(mobile): auth controller + login/register screens (brand-styled)"
```

---

### Task 8: Theme, router, app wiring

**TDD: no** — composition + theming; verify by launching the app.

**Files:** `lib/core/theme/app_theme.dart`, `lib/core/router/app_router.dart`, `lib/main.dart`. A placeholder `HomeScreen` (vehicle list lands in Plan 2).

**Invoke `frontend-design` + `color-expert` first** — encode the brand tokens.

- [ ] **Step 1: Create `lib/core/theme/app_theme.dart`**

```dart
import 'package:flutter/material.dart';

/// Veyra brand tokens (reference → semantic). Dark-first.
abstract class VeyraColors {
  static const accent = Color(0xFFF26A21);
  static const bg = Color(0xFF0D1119);
  static const surface = Color(0xFF151A23);
  static const text = Color(0xFFE6EAF0);
  static const textMuted = Color(0xFF9BA6B5);
}

ThemeData buildDarkTheme() {
  final scheme = ColorScheme.fromSeed(
    seedColor: VeyraColors.accent,
    brightness: Brightness.dark,
    surface: VeyraColors.surface,
  ).copyWith(primary: VeyraColors.accent);
  return ThemeData(
    useMaterial3: true,
    colorScheme: scheme,
    scaffoldBackgroundColor: VeyraColors.bg,
    textTheme: const TextTheme().apply(
      bodyColor: VeyraColors.text,
      displayColor: VeyraColors.text,
    ),
  );
}
```

- [ ] **Step 2: Create `lib/core/router/app_router.dart`**

```dart
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../features/auth/presentation/controllers/auth_controller.dart';
import '../../features/auth/presentation/screens/login_screen.dart';
import '../../features/auth/presentation/screens/register_screen.dart';

/// Placeholder home — replaced by the vehicle list in Plan 2.
class HomeScreen extends ConsumerWidget {
  const HomeScreen({super.key});
  @override
  Widget build(BuildContext context, WidgetRef ref) => Scaffold(
        appBar: AppBar(title: const Text('Veyra')),
        body: Center(
          child: TextButton(
            onPressed: () => ref.read(authControllerProvider.notifier).logout(),
            child: const Text('Log out'),
          ),
        ),
      );
}

final routerProvider = Provider<GoRouter>((ref) {
  return GoRouter(
    initialLocation: '/',
    redirect: (context, state) {
      final auth = ref.read(authControllerProvider).valueOrNull;
      final loggedIn = auth is Authenticated;
      final atAuth = state.matchedLocation == '/login' || state.matchedLocation == '/register';
      if (!loggedIn && !atAuth) return '/login';
      if (loggedIn && atAuth) return '/';
      return null;
    },
    routes: [
      GoRoute(path: '/', builder: (_, __) => const HomeScreen()),
      GoRoute(path: '/login', builder: (_, __) => const LoginScreen()),
      GoRoute(path: '/register', builder: (_, __) => const RegisterScreen()),
    ],
  );
});
```

- [ ] **Step 3: Create `lib/main.dart`**

```dart
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'core/router/app_router.dart';
import 'core/theme/app_theme.dart';

void main() => runApp(const ProviderScope(child: VeyraApp()));

class VeyraApp extends ConsumerWidget {
  const VeyraApp({super.key});
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final router = ref.watch(routerProvider);
    return MaterialApp.router(
      title: 'Veyra',
      debugShowCheckedModeBanner: false,
      theme: buildDarkTheme(),
      routerConfig: router,
    );
  }
}
```

> The router should re-evaluate when auth state changes. Add a `refreshListenable` bridging
> `authControllerProvider` (e.g. a `ValueNotifier` toggled in a `ref.listen`) OR call
> `router.refresh()` from a `ref.listen(authControllerProvider, ...)` in `VeyraApp`. Implement one of
> these so logout/login redirects fire — do not leave it implicit.

- [ ] **Step 4: Run the app end-to-end against the backend**

```bash
# Backend (bearer mode, ADR-0007) must be running on localhost:3000.
cd apps/mobile && flutter run --dart-define=API_BASE_URL=http://localhost:3000
# Manual: register → lands on Home; logout → back to /login; bad email → inline error.
```

- [ ] **Step 5: Full gate + commit**

```bash
flutter analyze && flutter test && dart format --set-exit-if-changed . && dart run tool/check_boundaries.dart
git add apps/mobile/lib/core/theme/ apps/mobile/lib/core/router/ apps/mobile/lib/main.dart
git commit -m "feat(mobile): brand theme, go_router auth guard, app wiring"
```

---

## Self-Review

- **Spec coverage:** layering + boundary tool (T1) · Failure + error mapping (T2) · token store + config (T3) · auth value objects (T4) · entity/port/use cases + dio interceptors incl. single-flight refresh (T5) · data layer DTO/datasource/repo + DI (T6) · controller + screens (T7) · theme/router/wiring (T8). Auth feature fully realized as the template; networking/auth-flow/theming from the spec covered.
- **Placeholder scan:** none — every step has real code or exact commands. `register_screen.dart` (T7 S3) and the router refresh bridge (T8 S3) are described with explicit content requirements, not "TODO".
- **Type consistency:** `Tokens`/`TokenStore` (T3) used by interceptors (T5) + repo (T6); `Failure`/`mapDioError` (T2) used in T6; `User`/`Email`/`Password` (T4/T5) used through T6/T7; provider names (`dioProvider`, `authRepositoryProvider`, `*UseCaseProvider`, `authControllerProvider`, `routerProvider`, `sessionExpiredProvider`) are consistent across tasks.
- **Error shape:** `mapDioError` reads flat `data['error']` (matches the backend).

---

## Next Plans (follow-on)

- **Plan 2:** `vehicle` feature (list/detail/create/edit/delete) + `dashboard` summary — replicates this template; adds the vehicle value objects (`PlateNumber`, `Odometer`, `FuelType`) and the detail-screen tab shell.
- **Plan 3:** `service_record` / `fuel_log` / `expense` / `reminder` / `document` — mechanical replication of the template per feature, with `Money`/`Decimal` value objects and the reminder cross-field rule.
