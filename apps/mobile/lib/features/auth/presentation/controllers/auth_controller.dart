import 'dart:async' show unawaited;

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/core/i18n/locale_controller.dart';
import 'package:veyra_mobile/core/network/auth_events.dart';
import 'package:veyra_mobile/core/storage/token_store.dart';
import 'package:veyra_mobile/features/auth/data/repositories/auth_repository_impl.dart';
import 'package:veyra_mobile/features/auth/domain/entities/user.dart';
import 'package:veyra_mobile/features/auth/domain/value_objects/email.dart';
import 'package:veyra_mobile/features/auth/domain/value_objects/password.dart';

/// Holds the authenticated [User] (or null = logged out). `AsyncValue` carries
/// the loading state during the initial session check and during login.
class AuthController extends AsyncNotifier<User?> {
  /// Keep the branded splash on screen at least this long so its open-animation
  /// (~1.05s) always plays in full, then holds briefly before navigating.
  static const _minSplash = Duration(milliseconds: 1500);

  @override
  Future<User?> build() async {
    // Network layer signals a hard session expiry → flip to logged out.
    ref.listen(authEventsProvider, (_, _) {
      state = const AsyncData(null);
    });

    final (user, _) =
        await (_restoreSession(), Future<void>.delayed(_minSplash)).wait;
    return user;
  }

  Future<User?> _restoreSession() async {
    final tokens = await ref.read(tokenStoreProvider).read();
    if (tokens == null) return null;
    final result = await ref.read(getMeUseCaseProvider)();
    return result.fold((_) => null, (user) {
      _adoptLanguage(user);
      return user;
    });
  }

  /// Returns the [Failure] to show inline, or null on success.
  Future<Failure?> login(Email email, Password password) async {
    state = const AsyncLoading();
    final result = await ref.read(loginUseCaseProvider)(email, password);
    return result.fold(
      (failure) {
        state = const AsyncData(null);
        return failure;
      },
      (user) {
        _adoptLanguage(user);
        state = AsyncData(user);
        return null;
      },
    );
  }

  /// Fire-and-forget: adopt the server language for the user if set.
  void _adoptLanguage(User user) {
    final lang = user.preferredLanguage;
    if (lang != null) {
      unawaited(
        ref
            .read(localeControllerProvider.notifier)
            .adoptBackendLanguage(lang),
      );
    }
  }

  Future<Failure?> register(Email email, Password password, String name) async {
    state = const AsyncLoading();
    final result = await ref.read(registerUseCaseProvider)(
      email,
      password,
      name,
    );
    return result.fold(
      (failure) {
        state = const AsyncData(null);
        return failure;
      },
      (user) {
        state = AsyncData(user);
        return null;
      },
    );
  }

  Future<void> logout() async {
    await ref.read(logoutUseCaseProvider)();
    state = const AsyncData(null);
  }
}

final authControllerProvider = AsyncNotifierProvider<AuthController, User?>(
  AuthController.new,
);
