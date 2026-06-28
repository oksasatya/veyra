import 'package:dio/dio.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:fpdart/fpdart.dart';
import 'package:veyra_mobile/core/error/dio_error_mapper.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/core/network/dio_client.dart';
import 'package:veyra_mobile/core/storage/token_store.dart';
import 'package:veyra_mobile/features/auth/data/datasources/auth_remote_data_source.dart';
import 'package:veyra_mobile/features/auth/domain/entities/user.dart';
import 'package:veyra_mobile/features/auth/domain/repositories/auth_repository.dart';
import 'package:veyra_mobile/features/auth/domain/usecases/get_me_usecase.dart';
import 'package:veyra_mobile/features/auth/domain/usecases/login_usecase.dart';
import 'package:veyra_mobile/features/auth/domain/usecases/logout_usecase.dart';
import 'package:veyra_mobile/features/auth/domain/usecases/register_usecase.dart';
import 'package:veyra_mobile/features/auth/domain/usecases/update_preferences_usecase.dart';
import 'package:veyra_mobile/features/auth/domain/value_objects/email.dart';
import 'package:veyra_mobile/features/auth/domain/value_objects/password.dart';

class AuthRepositoryImpl implements AuthRepository {
  AuthRepositoryImpl({required this.remote, required this.store});
  final AuthRemoteDataSource remote;
  final TokenStore store;

  @override
  Future<Either<Failure, User>> login({
    required Email email,
    required Password password,
  }) => _authCall(() => remote.login(email.value, password.value));

  @override
  Future<Either<Failure, User>> register({
    required Email email,
    required Password password,
    required String name,
  }) => _authCall(() => remote.register(email.value, password.value, name));

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

  @override
  Future<Either<Failure, void>> updatePreferences(String language) async {
    try {
      await remote.updatePreferences(language);
      return right(null);
    } on DioException catch (e) {
      return left(mapDioError(e));
    }
  }

  Future<Either<Failure, User>> _authCall(
    Future<AuthPayload> Function() call,
  ) async {
    try {
      final result = await call();
      await store.save(result.tokens.toStore());
      return Right(result.user.toDomain());
    } on DioException catch (e) {
      return Left(mapDioError(e));
    }
  }
}

// ── Providers (DI) ───────────────────────────────────────────────────────────

final authRepositoryProvider = Provider<AuthRepository>(
  (ref) => AuthRepositoryImpl(
    remote: AuthRemoteDataSource(ref.watch(dioProvider)),
    store: ref.watch(tokenStoreProvider),
  ),
);

final loginUseCaseProvider = Provider<LoginUseCase>(
  (ref) => LoginUseCase(ref.watch(authRepositoryProvider)),
);
final registerUseCaseProvider = Provider<RegisterUseCase>(
  (ref) => RegisterUseCase(ref.watch(authRepositoryProvider)),
);
final logoutUseCaseProvider = Provider<LogoutUseCase>(
  (ref) => LogoutUseCase(ref.watch(authRepositoryProvider)),
);
final getMeUseCaseProvider = Provider<GetMeUseCase>(
  (ref) => GetMeUseCase(ref.watch(authRepositoryProvider)),
);
final updatePreferencesUseCaseProvider = Provider<UpdatePreferencesUseCase>(
  (ref) => UpdatePreferencesUseCase(ref.watch(authRepositoryProvider)),
);
