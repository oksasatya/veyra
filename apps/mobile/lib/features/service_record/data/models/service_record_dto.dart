import 'package:decimal/decimal.dart';
import 'package:veyra_mobile/features/service_record/domain/entities/service_record.dart';

/// Wire model for `ServiceRecordResponse`. Hand-mapped to [ServiceRecord].
/// `cost` is a nullable monetary string → `Decimal?`; `service_date` is an ISO
/// date → `DateTime`.
class ServiceRecordDto {
  const ServiceRecordDto({
    required this.id,
    required this.vehicleId,
    required this.serviceDate,
    required this.odometer,
    required this.description,
    this.workshop,
    this.cost,
    this.notes,
  });

  factory ServiceRecordDto.fromJson(Map<String, dynamic> json) =>
      ServiceRecordDto(
        id: json['id'] as String,
        vehicleId: json['vehicle_id'] as String,
        serviceDate: json['service_date'] as String,
        odometer: json['odometer'] as int,
        description: json['description'] as String,
        workshop: json['workshop'] as String?,
        cost: json['cost'] as String?,
        notes: json['notes'] as String?,
      );

  final String id;
  final String vehicleId;
  final String serviceDate;
  final int odometer;
  final String description;
  final String? workshop;
  final String? cost;
  final String? notes;

  ServiceRecord toDomain() => ServiceRecord(
        id: id,
        vehicleId: vehicleId,
        serviceDate: DateTime.parse(serviceDate),
        odometer: odometer,
        description: description,
        workshop: workshop,
        cost: cost == null ? null : Decimal.parse(cost!),
        notes: notes,
      );
}
