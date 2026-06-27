import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/features/vehicle/data/repositories/vehicle_repository_impl.dart';
import 'package:veyra_mobile/features/vehicle/domain/entities/vehicle.dart';
import 'package:veyra_mobile/features/vehicle/domain/repositories/vehicle_repository.dart';

/// Loads the authenticated user's vehicles. `AsyncValue` carries loading/error;
/// a thrown [Failure] surfaces as the error state for the UI.
class GarageController extends AsyncNotifier<List<Vehicle>> {
  @override
  Future<List<Vehicle>> build() => _load();

  Future<List<Vehicle>> _load() async {
    final result = await ref.read(listVehiclesUseCaseProvider)();
    return result.fold((failure) => throw failure, (vehicles) => vehicles);
  }

  Future<void> refresh() async {
    state = const AsyncLoading();
    state = await AsyncValue.guard(_load);
  }

  /// Returns the [Failure] to show inline, or null on success (list refreshed).
  Future<Failure?> add(CreateVehicleInput input) async {
    final result = await ref.read(createVehicleUseCaseProvider)(input);
    if (result.isLeft()) {
      return result.getLeft().toNullable();
    }
    await refresh();
    return null;
  }
}

final garageControllerProvider =
    AsyncNotifierProvider<GarageController, List<Vehicle>>(GarageController.new);
