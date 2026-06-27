import 'package:fpdart/fpdart.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/features/vehicle/domain/entities/vehicle.dart';
import 'package:veyra_mobile/features/vehicle/domain/value_objects/fuel_type.dart';

/// Validated input for creating a vehicle (the use case builds this from VOs).
class CreateVehicleInput {
  const CreateVehicleInput({
    required this.brand,
    required this.model,
    required this.year,
    required this.plateNumber,
    required this.fuelType,
    required this.odometer,
    this.color,
    this.notes,
  });

  final String brand;
  final String model;
  final int year;
  final String plateNumber;
  final FuelType fuelType;
  final int odometer;
  final String? color;
  final String? notes;
}

/// Port: the vehicle boundary the domain depends on.
abstract interface class VehicleRepository {
  Future<Either<Failure, List<Vehicle>>> list();
  Future<Either<Failure, Vehicle>> create(CreateVehicleInput input);
}
