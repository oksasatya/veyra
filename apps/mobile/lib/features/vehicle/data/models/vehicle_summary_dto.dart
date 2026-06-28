import 'package:decimal/decimal.dart';
import 'package:veyra_mobile/features/vehicle/domain/entities/vehicle_summary.dart';

/// Wire model for `VehicleSummaryResponse`. Monetary fields arrive as strings.
class VehicleSummaryDto {
  const VehicleSummaryDto({
    required this.currentOdometer,
    required this.totalServices,
    required this.totalServiceCost,
    required this.totalRefuels,
    required this.totalFuelCost,
    required this.totalExpenses,
    required this.upcomingReminders,
  });

  factory VehicleSummaryDto.fromJson(Map<String, dynamic> json) =>
      VehicleSummaryDto(
        currentOdometer: json['current_odometer'] as int,
        totalServices: json['total_services'] as int,
        totalServiceCost: json['total_service_cost'] as String,
        totalRefuels: json['total_refuels'] as int,
        totalFuelCost: json['total_fuel_cost'] as String,
        totalExpenses: json['total_expenses'] as String,
        upcomingReminders: json['upcoming_reminders'] as int,
      );

  final int currentOdometer;
  final int totalServices;
  final String totalServiceCost;
  final int totalRefuels;
  final String totalFuelCost;
  final String totalExpenses;
  final int upcomingReminders;

  VehicleSummary toDomain() => VehicleSummary(
        currentOdometer: currentOdometer,
        totalServices: totalServices,
        totalServiceCost: Decimal.parse(totalServiceCost),
        totalRefuels: totalRefuels,
        totalFuelCost: Decimal.parse(totalFuelCost),
        totalExpenses: Decimal.parse(totalExpenses),
        upcomingReminders: upcomingReminders,
      );
}
