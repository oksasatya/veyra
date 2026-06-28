import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:veyra_mobile/features/reminder/data/repositories/reminder_repository_impl.dart';
import 'package:veyra_mobile/features/reminder/domain/entities/reminder.dart';
import 'package:veyra_mobile/features/vehicle/presentation/controllers/garage_dashboard_controller.dart';

/// A reminder paired with the display name of the vehicle it belongs to, so the
/// cross-vehicle list can show "Toyota Avanza · due 18 Jun" without a re-lookup.
class ReminderWithVehicle {
  const ReminderWithVehicle(this.reminder, this.vehicleName);
  final Reminder reminder;
  final String vehicleName;
}

/// All reminders across every vehicle, flattened. Depends on the garage
/// dashboard for the vehicle set, then fans out one reminders read per vehicle
/// in parallel — O(V) bounded reads for a personal garage (a handful of cars).
final remindersOverviewProvider =
    FutureProvider<List<ReminderWithVehicle>>((ref) async {
  final dashboard = await ref.watch(garageDashboardProvider.future);
  final perVehicle = await Future.wait(
    dashboard.entries.map((entry) async {
      final reminders =
          await ref.watch(reminderListProvider(entry.vehicle.id).future);
      return reminders
          .map((r) => ReminderWithVehicle(r, entry.vehicle.displayName))
          .toList();
    }),
  );
  return perVehicle.expand((list) => list).toList();
});
