import 'package:decimal/decimal.dart';
import 'package:fpdart/fpdart.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/features/service_record/domain/entities/service_record.dart';

/// Validated input for creating a service record.
class CreateServiceRecordInput {
  const CreateServiceRecordInput({
    required this.vehicleId,
    required this.serviceDate,
    required this.odometer,
    required this.description,
    this.workshop,
    this.cost,
    this.notes,
  });

  final String vehicleId;
  final DateTime serviceDate;
  final int odometer;
  final String description;
  final String? workshop;
  final Decimal? cost;
  final String? notes;
}

/// Port: the service-record boundary the domain depends on.
abstract interface class ServiceRecordRepository {
  Future<Either<Failure, List<ServiceRecord>>> list(String vehicleId);
  Future<Either<Failure, ServiceRecord>> create(CreateServiceRecordInput input);
}
