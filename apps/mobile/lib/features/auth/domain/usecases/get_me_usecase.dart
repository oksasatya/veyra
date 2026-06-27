import 'package:fpdart/fpdart.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/features/auth/domain/entities/user.dart';
import 'package:veyra_mobile/features/auth/domain/repositories/auth_repository.dart';

class GetMeUseCase {
  const GetMeUseCase(this._repo);
  final AuthRepository _repo;

  Future<Either<Failure, User>> call() => _repo.getMe();
}
