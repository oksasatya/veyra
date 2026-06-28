import 'package:fpdart/fpdart.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/features/auth/domain/repositories/auth_repository.dart';

class UpdatePreferencesUseCase {
  const UpdatePreferencesUseCase(this._repo);
  final AuthRepository _repo;

  Future<Either<Failure, void>> call(String language) =>
      _repo.updatePreferences(language);
}
