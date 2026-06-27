import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/core/network/auth_events.dart';
import 'package:veyra_mobile/core/storage/token_store.dart';
import 'package:veyra_mobile/features/auth/data/repositories/auth_repository_impl.dart';
import 'package:veyra_mobile/features/auth/domain/entities/user.dart';
import 'package:veyra_mobile/features/auth/domain/value_objects/email.dart';
import 'package:veyra_mobile/features/auth/domain/value_objects/password.dart';

/// Holds the authenticated [User] (or null = logged out). `AsyncValue` carries
/// the loading state during the initial session check and during login.
class AuthController extends AsyncNotifier<User?> {
  @override
  Future<User?> build() async {
    // Network layer signals a hard session expiry → flip to logged out.
    ref.listen(authEventsProvider, (_, _) {
      state = const AsyncData(null);
    });

    final tokens = await ref.read(tokenStoreProvider).read();
    if (tokens == null) return null;
    final result = await ref.read(getMeUseCaseProvider)();
    return result.fold((_) => null, (user) => user);
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
        state = AsyncData(user);
        return null;
      },
    );
  }

  Future<Failure?> register(Email email, Password password, String name) async {
    state = const AsyncLoading();
    final result = await ref.read(registerUseCaseProvider)(email, password, name);
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

final authControllerProvider =
    AsyncNotifierProvider<AuthController, User?>(AuthController.new);
