import 'package:decimal/decimal.dart';
import 'package:veyra_mobile/features/fuel_log/domain/entities/fuel_log.dart';

/// Wire model for `FuelLogResponse`. Hand-mapped to the [FuelLog] entity.
/// Monetary fields (`liters`, `price_per_liter`, `total_cost`) arrive as strings
/// to preserve precision and are parsed into [Decimal].
class FuelLogDto {
  const FuelLogDto({
    required this.id,
    required this.vehicleId,
    required this.logDate,
    required this.odometer,
    required this.liters,
    required this.pricePerLiter,
    required this.totalCost,
    required this.isFullTank,
    this.station,
  });

  factory FuelLogDto.fromJson(Map<String, dynamic> json) => FuelLogDto(
    id: json['id'] as String,
    vehicleId: json['vehicle_id'] as String,
    logDate: json['log_date'] as String,
    odometer: json['odometer'] as int,
    liters: json['liters'] as String,
    pricePerLiter: json['price_per_liter'] as String,
    totalCost: json['total_cost'] as String,
    station: json['station'] as String?,
    isFullTank: json['is_full_tank'] as bool,
  );

  final String id;
  final String vehicleId;
  final String logDate;
  final int odometer;
  final String liters;
  final String pricePerLiter;
  final String totalCost;
  final String? station;
  final bool isFullTank;

  FuelLog toDomain() => FuelLog(
    id: id,
    vehicleId: vehicleId,
    logDate: DateTime.parse(logDate),
    odometer: odometer,
    liters: Decimal.parse(liters),
    pricePerLiter: Decimal.parse(pricePerLiter),
    totalCost: Decimal.parse(totalCost),
    station: station,
    isFullTank: isFullTank,
  );
}
