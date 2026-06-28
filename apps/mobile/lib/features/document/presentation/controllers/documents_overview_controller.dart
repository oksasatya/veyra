import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:veyra_mobile/features/document/data/repositories/document_repository_impl.dart';
import 'package:veyra_mobile/features/document/domain/entities/document.dart';
import 'package:veyra_mobile/features/vehicle/presentation/controllers/garage_dashboard_controller.dart';

/// A document paired with its owning vehicle's display name for the
/// cross-vehicle list ("Toyota Avanza · Expires 18 Jun 2026").
class DocumentWithVehicle {
  const DocumentWithVehicle(this.document, this.vehicleName);
  final Document document;
  final String vehicleName;
}

/// Every document across all vehicles, flattened. Mirrors the reminders
/// overview: vehicle set from the garage dashboard, then one documents read per
/// vehicle in parallel — O(V) bounded reads for a personal garage.
final documentsOverviewProvider =
    FutureProvider<List<DocumentWithVehicle>>((ref) async {
  final dashboard = await ref.watch(garageDashboardProvider.future);
  final perVehicle = await Future.wait(
    dashboard.entries.map((entry) async {
      final docs =
          await ref.watch(documentListProvider(entry.vehicle.id).future);
      return docs
          .map((d) => DocumentWithVehicle(d, entry.vehicle.displayName))
          .toList();
    }),
  );
  return perVehicle.expand((list) => list).toList();
});
