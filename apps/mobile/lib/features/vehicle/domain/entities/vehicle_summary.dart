import 'package:decimal/decimal.dart';

/// Per-vehicle dashboard aggregate (from GET /vehicles/:id/summary).
/// Monetary fields are [Decimal] — the API serialises them as strings to avoid
/// floating-point loss.
class VehicleSummary {
  const VehicleSummary({
    required this.currentOdometer,
    required this.totalServices,
    required this.totalServiceCost,
    required this.totalRefuels,
    required this.totalFuelCost,
    required this.totalExpenses,
    required this.upcomingReminders,
  });

  final int currentOdometer;
  final int totalServices;
  final Decimal totalServiceCost;
  final int totalRefuels;
  final Decimal totalFuelCost;
  final Decimal totalExpenses;
  final int upcomingReminders;
}
