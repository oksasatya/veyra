import 'package:fpdart/fpdart.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/features/vehicle/domain/entities/vehicle.dart';
import 'package:veyra_mobile/features/vehicle/domain/repositories/vehicle_repository.dart';

class CreateVehicleUseCase {
  const CreateVehicleUseCase(this._repo);
  final VehicleRepository _repo;

  Future<Either<Failure, Vehicle>> call(CreateVehicleInput input) =>
      _repo.create(input);
}
