import 'package:dio/dio.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:fpdart/fpdart.dart';
import 'package:veyra_mobile/core/error/dio_error_mapper.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/core/network/dio_client.dart';
import 'package:veyra_mobile/features/fuel_log/data/datasources/fuel_log_remote_data_source.dart';
import 'package:veyra_mobile/features/fuel_log/domain/entities/fuel_log.dart';
import 'package:veyra_mobile/features/fuel_log/domain/repositories/fuel_log_repository.dart';
import 'package:veyra_mobile/features/fuel_log/domain/usecases/create_fuel_log_usecase.dart';
import 'package:veyra_mobile/features/fuel_log/domain/usecases/list_fuel_logs_usecase.dart';

class FuelLogRepositoryImpl implements FuelLogRepository {
  FuelLogRepositoryImpl(this.remote);
  final FuelLogRemoteDataSource remote;

  @override
  Future<Either<Failure, List<FuelLog>>> list(String vehicleId) async {
    try {
      final dtos = await remote.list(vehicleId);
      return Right(dtos.map((d) => d.toDomain()).toList());
    } on DioException catch (e) {
      return Left(mapDioError(e));
    }
  }

  @override
  Future<Either<Failure, FuelLog>> create(CreateFuelLogInput input) async {
    try {
      final dto = await remote.create(input.vehicleId, {
        'log_date': input.logDate.toIso8601String().split('T').first,
        'odometer': input.odometer,
        'liters': input.liters.toString(),
        'price_per_liter': input.pricePerLiter.toString(),
        'station': input.station,
        'is_full_tank': input.isFullTank,
      });
      return Right(dto.toDomain());
    } on DioException catch (e) {
      return Left(mapDioError(e));
    }
  }
}

// ── Providers (DI) ───────────────────────────────────────────────────────────

final fuelLogRepositoryProvider = Provider<FuelLogRepository>(
  (ref) =>
      FuelLogRepositoryImpl(FuelLogRemoteDataSource(ref.watch(dioProvider))),
);

final listFuelLogsUseCaseProvider = Provider<ListFuelLogsUseCase>(
  (ref) => ListFuelLogsUseCase(ref.watch(fuelLogRepositoryProvider)),
);

final createFuelLogUseCaseProvider = Provider<CreateFuelLogUseCase>(
  (ref) => CreateFuelLogUseCase(ref.watch(fuelLogRepositoryProvider)),
);

/// Per-vehicle fuel logs, keyed by vehicle id. The add-sheet invalidates this
/// after a successful create so the list refreshes.
final fuelLogListProvider = FutureProvider.family<List<FuelLog>, String>((
  ref,
  vehicleId,
) async {
  final result = await ref.read(listFuelLogsUseCaseProvider)(vehicleId);
  return result.fold((failure) => throw failure, (logs) => logs);
});
