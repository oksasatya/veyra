import 'package:decimal/decimal.dart';
import 'package:fpdart/fpdart.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/features/fuel_log/domain/entities/fuel_log.dart';

/// Validated input for creating a fuel log (the sheet builds this from VOs).
class CreateFuelLogInput {
  const CreateFuelLogInput({
    required this.vehicleId,
    required this.logDate,
    required this.odometer,
    required this.liters,
    required this.pricePerLiter,
    required this.isFullTank,
    this.station,
  });

  final String vehicleId;
  final DateTime logDate;
  final int odometer;
  final Decimal liters;
  final Decimal pricePerLiter;
  final String? station;
  final bool isFullTank;
}

/// Port: the fuel-log boundary the domain depends on.
abstract interface class FuelLogRepository {
  Future<Either<Failure, List<FuelLog>>> list(String vehicleId);
  Future<Either<Failure, FuelLog>> create(CreateFuelLogInput input);
}
