import 'package:fpdart/fpdart.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/features/auth/domain/entities/user.dart';
import 'package:veyra_mobile/features/auth/domain/value_objects/email.dart';
import 'package:veyra_mobile/features/auth/domain/value_objects/password.dart';

/// Port: the auth boundary the domain depends on. Implemented in the data layer.
abstract interface class AuthRepository {
  Future<Either<Failure, User>> login({
    required Email email,
    required Password password,
  });

  Future<Either<Failure, User>> register({
    required Email email,
    required Password password,
    required String name,
  });

  Future<Either<Failure, Unit>> logout();

  Future<Either<Failure, User>> getMe();

  Future<Either<Failure, void>> updatePreferences(String language);
}
