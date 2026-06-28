import 'package:decimal/decimal.dart';

/// Domain entity — pure Dart. The data layer maps a DTO into this.
class FuelLog {
  const FuelLog({
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

  final String id;
  final String vehicleId;
  final DateTime logDate;
  final int odometer;
  final Decimal liters;
  final Decimal pricePerLiter;
  final Decimal totalCost;
  final String? station;
  final bool isFullTank;
}
