import 'package:fpdart/fpdart.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/features/fuel_log/domain/entities/fuel_log.dart';
import 'package:veyra_mobile/features/fuel_log/domain/repositories/fuel_log_repository.dart';

class ListFuelLogsUseCase {
  const ListFuelLogsUseCase(this._repo);
  final FuelLogRepository _repo;

  Future<Either<Failure, List<FuelLog>>> call(String vehicleId) =>
      _repo.list(vehicleId);
}
