import 'package:fpdart/fpdart.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/features/service_record/domain/entities/service_record.dart';
import 'package:veyra_mobile/features/service_record/domain/repositories/service_record_repository.dart';

class CreateServiceRecordUseCase {
  const CreateServiceRecordUseCase(this._repo);
  final ServiceRecordRepository _repo;

  Future<Either<Failure, ServiceRecord>> call(
    CreateServiceRecordInput input,
  ) => _repo.create(input);
}
