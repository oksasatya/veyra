import 'package:fpdart/fpdart.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/features/service_record/domain/entities/service_record.dart';
import 'package:veyra_mobile/features/service_record/domain/repositories/service_record_repository.dart';

class ListServiceRecordsUseCase {
  const ListServiceRecordsUseCase(this._repo);
  final ServiceRecordRepository _repo;

  Future<Either<Failure, List<ServiceRecord>>> call(String vehicleId) =>
      _repo.list(vehicleId);
}
