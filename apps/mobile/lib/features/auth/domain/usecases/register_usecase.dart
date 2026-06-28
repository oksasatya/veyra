import 'package:fpdart/fpdart.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/features/auth/domain/entities/user.dart';
import 'package:veyra_mobile/features/auth/domain/repositories/auth_repository.dart';
import 'package:veyra_mobile/features/auth/domain/value_objects/email.dart';
import 'package:veyra_mobile/features/auth/domain/value_objects/password.dart';

class RegisterUseCase {
  const RegisterUseCase(this._repo);
  final AuthRepository _repo;

  Future<Either<Failure, User>> call(
    Email email,
    Password password,
    String name,
  ) => _repo.register(email: email, password: password, name: name);
}
