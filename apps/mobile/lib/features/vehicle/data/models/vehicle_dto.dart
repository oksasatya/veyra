import 'package:veyra_mobile/features/vehicle/domain/entities/vehicle.dart';
import 'package:veyra_mobile/features/vehicle/domain/value_objects/fuel_type.dart';

/// Wire model for `VehicleResponse`. Hand-mapped to the [Vehicle] entity.
class VehicleDto {
  const VehicleDto({
    required this.id,
    required this.brand,
    required this.model,
    required this.year,
    required this.plateNumber,
    required this.fuelType,
    required this.currentOdometer,
    this.color,
    this.notes,
  });

  factory VehicleDto.fromJson(Map<String, dynamic> json) => VehicleDto(
    id: json['id'] as String,
    brand: json['brand'] as String,
    model: json['model'] as String,
    year: json['year'] as int,
    plateNumber: json['plate_number'] as String,
    fuelType: json['fuel_type'] as String,
    currentOdometer: json['current_odometer'] as int,
    color: json['color'] as String?,
    notes: json['notes'] as String?,
  );

  final String id;
  final String brand;
  final String model;
  final int year;
  final String plateNumber;
  final String fuelType;
  final int currentOdometer;
  final String? color;
  final String? notes;

  Vehicle toDomain() => Vehicle(
    id: id,
    brand: brand,
    model: model,
    year: year,
    plateNumber: plateNumber,
    fuelType: FuelType.fromApi(fuelType),
    odometer: currentOdometer,
    color: color,
    notes: notes,
  );
}
