import 'package:fpdart/fpdart.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/features/fuel_log/domain/entities/fuel_log.dart';
import 'package:veyra_mobile/features/fuel_log/domain/repositories/fuel_log_repository.dart';

class CreateFuelLogUseCase {
  const CreateFuelLogUseCase(this._repo);
  final FuelLogRepository _repo;

  Future<Either<Failure, FuelLog>> call(CreateFuelLogInput input) =>
      _repo.create(input);
}
