import 'package:decimal/decimal.dart';

/// Domain entity — pure Dart. The data layer maps a DTO into this.
class ServiceRecord {
  const ServiceRecord({
    required this.id,
    required this.vehicleId,
    required this.serviceDate,
    required this.odometer,
    required this.description,
    this.workshop,
    this.cost,
    this.notes,
  });

  final String id;
  final String vehicleId;
  final DateTime serviceDate;
  final int odometer;
  final String description;
  final String? workshop;
  final Decimal? cost;
  final String? notes;
}
