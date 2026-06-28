import 'package:decimal/decimal.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:veyra_mobile/features/vehicle/data/repositories/vehicle_repository_impl.dart';
import 'package:veyra_mobile/features/vehicle/domain/entities/vehicle.dart';
import 'package:veyra_mobile/features/vehicle/domain/entities/vehicle_summary.dart';

/// One garage row: a vehicle paired with its dashboard summary.
class GarageEntry {
  const GarageEntry({required this.vehicle, required this.summary});
  final Vehicle vehicle;
  final VehicleSummary summary;

  Decimal get spent =>
      summary.totalServiceCost + summary.totalFuelCost + summary.totalExpenses;
}

/// The garage dashboard view model: per-vehicle entries + cross-vehicle totals.
class GarageDashboard {
  const GarageDashboard(this.entries);
  final List<GarageEntry> entries;

  bool get isEmpty => entries.isEmpty;
  int get vehicleCount => entries.length;
  int get dueSoon =>
      entries.fold(0, (sum, e) => sum + e.summary.upcomingReminders);
  Decimal get totalSpent =>
      entries.fold(Decimal.zero, (sum, e) => sum + e.spent);
}

/// Loads vehicles, then their summaries in parallel (Future.wait → wall-clock is
/// the slowest single summary call, not the sum). N+1 by request count, bounded
/// for a personal garage; a batch endpoint would flatten it if N ever grows.
final garageDashboardProvider = FutureProvider<GarageDashboard>((ref) async {
  final vehiclesResult = await ref.read(listVehiclesUseCaseProvider)();
  final vehicles = vehiclesResult.fold<List<Vehicle>>(
    (failure) => throw failure,
    (list) => list,
  );
  if (vehicles.isEmpty) return const GarageDashboard([]);

  final summaries = await Future.wait(
    vehicles.map((v) async {
      final result = await ref.read(getSummaryUseCaseProvider)(v.id);
      return result.fold<VehicleSummary>(
        (failure) => throw failure,
        (summary) => summary,
      );
    }),
  );

  return GarageDashboard([
    for (var i = 0; i < vehicles.length; i++)
      GarageEntry(vehicle: vehicles[i], summary: summaries[i]),
  ]);
});
