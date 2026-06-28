import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:veyra_mobile/features/expense/domain/entities/expense.dart';
import 'package:veyra_mobile/features/expense/presentation/controllers/expense_list_controller.dart';
import 'package:veyra_mobile/features/fuel_log/data/repositories/fuel_log_repository_impl.dart';
import 'package:veyra_mobile/features/fuel_log/domain/entities/fuel_log.dart';
import 'package:veyra_mobile/features/service_record/data/repositories/service_record_repository_impl.dart';
import 'package:veyra_mobile/features/service_record/domain/entities/service_record.dart';

/// One entry in the vehicle Overview activity feed. Sealed so the widget can
/// render each kind from its source entity (and localize at the call site)
/// while the merge stays pure.
sealed class ActivityItem {
  const ActivityItem(this.date);
  final DateTime date;
}

class FuelActivity extends ActivityItem {
  FuelActivity(this.log) : super(log.logDate);
  final FuelLog log;
}

class ServiceActivity extends ActivityItem {
  ServiceActivity(this.record) : super(record.serviceDate);
  final ServiceRecord record;
}

class ExpenseActivity extends ActivityItem {
  ExpenseActivity(this.expense) : super(expense.expenseDate);
  final Expense expense;
}

/// Merge the three activity streams into one newest-first timeline, capped at
/// [limit]. O(n) to combine + O(n log n) to sort, where n is the total entry
/// count already held in memory — no extra queries. Pure: no I/O, no Flutter.
List<ActivityItem> mergeActivity({
  required List<FuelLog> fuel,
  required List<ServiceRecord> services,
  required List<Expense> expenses,
  int limit = 8,
}) {
  final items = <ActivityItem>[
    ...fuel.map(FuelActivity.new),
    ...services.map(ServiceActivity.new),
    ...expenses.map(ExpenseActivity.new),
  ]..sort((a, b) => b.date.compareTo(a.date));
  return items.length > limit ? items.sublist(0, limit) : items;
}

/// Cross-type recent activity for one vehicle. Fans out the three per-vehicle
/// list reads in parallel (bounded — three reads), then merges. Mirrors the
/// reminders/documents overview fan-out pattern.
final vehicleActivityProvider =
    FutureProvider.family<List<ActivityItem>, String>((ref, vehicleId) async {
  final (fuel, services, expenses) = await (
    ref.watch(fuelLogListProvider(vehicleId).future),
    ref.watch(serviceRecordListProvider(vehicleId).future),
    ref.watch(expenseListProvider(vehicleId).future),
  ).wait;
  return mergeActivity(fuel: fuel, services: services, expenses: expenses);
});
