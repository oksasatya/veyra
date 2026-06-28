import 'package:dio/dio.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:fpdart/fpdart.dart';
import 'package:veyra_mobile/core/error/dio_error_mapper.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/core/network/dio_client.dart';
import 'package:veyra_mobile/features/service_record/data/datasources/service_record_remote_data_source.dart';
import 'package:veyra_mobile/features/service_record/domain/entities/service_record.dart';
import 'package:veyra_mobile/features/service_record/domain/repositories/service_record_repository.dart';
import 'package:veyra_mobile/features/service_record/domain/usecases/create_service_record_usecase.dart';
import 'package:veyra_mobile/features/service_record/domain/usecases/list_service_records_usecase.dart';

class ServiceRecordRepositoryImpl implements ServiceRecordRepository {
  ServiceRecordRepositoryImpl(this.remote);
  final ServiceRecordRemoteDataSource remote;

  @override
  Future<Either<Failure, List<ServiceRecord>>> list(String vehicleId) async {
    try {
      final dtos = await remote.list(vehicleId);
      return Right(dtos.map((d) => d.toDomain()).toList());
    } on DioException catch (e) {
      return Left(mapDioError(e));
    }
  }

  @override
  Future<Either<Failure, ServiceRecord>> create(
    CreateServiceRecordInput input,
  ) async {
    try {
      final dto = await remote.create(input.vehicleId, {
        'service_date': _isoDate(input.serviceDate),
        'odometer': input.odometer,
        'description': input.description,
        if (input.workshop != null) 'workshop': input.workshop,
        if (input.cost != null) 'cost': input.cost.toString(),
        if (input.notes != null) 'notes': input.notes,
      });
      return Right(dto.toDomain());
    } on DioException catch (e) {
      return Left(mapDioError(e));
    }
  }
}

/// `YYYY-MM-DD` — the backend's service_date is a calendar date, not a timestamp.
String _isoDate(DateTime d) =>
    '${d.year.toString().padLeft(4, '0')}-'
    '${d.month.toString().padLeft(2, '0')}-'
    '${d.day.toString().padLeft(2, '0')}';

// ── Providers (DI) ───────────────────────────────────────────────────────────

final serviceRecordRepositoryProvider = Provider<ServiceRecordRepository>(
  (ref) => ServiceRecordRepositoryImpl(
    ServiceRecordRemoteDataSource(ref.watch(dioProvider)),
  ),
);

final listServiceRecordsUseCaseProvider = Provider<ListServiceRecordsUseCase>(
  (ref) =>
      ListServiceRecordsUseCase(ref.watch(serviceRecordRepositoryProvider)),
);

final createServiceRecordUseCaseProvider = Provider<CreateServiceRecordUseCase>(
  (ref) =>
      CreateServiceRecordUseCase(ref.watch(serviceRecordRepositoryProvider)),
);

/// Per-vehicle service records, keyed by vehicle id.
final serviceRecordListProvider =
    FutureProvider.family<List<ServiceRecord>, String>((ref, vehicleId) async {
      final result = await ref.read(listServiceRecordsUseCaseProvider)(
        vehicleId,
      );
      return result.fold((failure) => throw failure, (records) => records);
    });
