import 'package:fpdart/fpdart.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/features/vehicle/domain/entities/vehicle_summary.dart';
import 'package:veyra_mobile/features/vehicle/domain/repositories/vehicle_repository.dart';

class GetSummaryUseCase {
  const GetSummaryUseCase(this._repo);
  final VehicleRepository _repo;

  Future<Either<Failure, VehicleSummary>> call(String vehicleId) =>
      _repo.summary(vehicleId);
}
