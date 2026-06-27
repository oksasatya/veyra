import 'package:dio/dio.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:fpdart/fpdart.dart';
import 'package:veyra_mobile/core/error/dio_error_mapper.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/core/network/dio_client.dart';
import 'package:veyra_mobile/features/vehicle/data/datasources/vehicle_remote_data_source.dart';
import 'package:veyra_mobile/features/vehicle/domain/entities/vehicle.dart';
import 'package:veyra_mobile/features/vehicle/domain/repositories/vehicle_repository.dart';
import 'package:veyra_mobile/features/vehicle/domain/usecases/create_vehicle_usecase.dart';
import 'package:veyra_mobile/features/vehicle/domain/usecases/list_vehicles_usecase.dart';

class VehicleRepositoryImpl implements VehicleRepository {
  VehicleRepositoryImpl(this.remote);
  final VehicleRemoteDataSource remote;

  @override
  Future<Either<Failure, List<Vehicle>>> list() async {
    try {
      final dtos = await remote.list();
      return Right(dtos.map((d) => d.toDomain()).toList());
    } on DioException catch (e) {
      return Left(mapDioError(e));
    }
  }

  @override
  Future<Either<Failure, Vehicle>> create(CreateVehicleInput input) async {
    try {
      final dto = await remote.create({
        'brand': input.brand,
        'model': input.model,
        'year': input.year,
        'plate_number': input.plateNumber,
        'color': input.color,
        'fuel_type': input.fuelType.apiValue,
        'current_odometer': input.odometer,
        'notes': input.notes,
      });
      return Right(dto.toDomain());
    } on DioException catch (e) {
      return Left(mapDioError(e));
    }
  }
}

// ── Providers (DI) ───────────────────────────────────────────────────────────

final vehicleRepositoryProvider = Provider<VehicleRepository>(
  (ref) => VehicleRepositoryImpl(VehicleRemoteDataSource(ref.watch(dioProvider))),
);

final listVehiclesUseCaseProvider = Provider<ListVehiclesUseCase>(
  (ref) => ListVehiclesUseCase(ref.watch(vehicleRepositoryProvider)),
);

final createVehicleUseCaseProvider = Provider<CreateVehicleUseCase>(
  (ref) => CreateVehicleUseCase(ref.watch(vehicleRepositoryProvider)),
);
