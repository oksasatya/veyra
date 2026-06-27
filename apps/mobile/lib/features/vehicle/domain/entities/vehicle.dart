import 'package:veyra_mobile/features/vehicle/domain/value_objects/fuel_type.dart';

/// Domain entity — pure Dart. The data layer maps a DTO into this.
class Vehicle {
  const Vehicle({
    required this.id,
    required this.brand,
    required this.model,
    required this.year,
    required this.plateNumber,
    required this.fuelType,
    required this.odometer,
    this.color,
    this.notes,
  });

  final String id;
  final String brand;
  final String model;
  final int year;
  final String plateNumber;
  final FuelType fuelType;
  final int odometer;
  final String? color;
  final String? notes;

  String get displayName => '$brand $model';
}
