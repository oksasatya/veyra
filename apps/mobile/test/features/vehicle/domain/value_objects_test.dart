import 'package:flutter_test/flutter_test.dart';
import 'package:veyra_mobile/features/vehicle/domain/value_objects/fuel_type.dart';
import 'package:veyra_mobile/features/vehicle/domain/value_objects/odometer.dart';
import 'package:veyra_mobile/features/vehicle/domain/value_objects/plate_number.dart';

void main() {
  group('PlateNumber', () {
    test('trims and upper-cases', () {
      final p = PlateNumber.create('  b 1234 xyz ').toNullable();
      expect(p?.value, 'B 1234 XYZ');
    });

    test('rejects empty', () {
      expect(PlateNumber.create('   ').isLeft(), isTrue);
    });
  });

  group('Odometer', () {
    test('accepts non-negative', () {
      expect(Odometer.create(50000).toNullable()?.value, 50000);
    });

    test('rejects negative', () {
      expect(Odometer.create(-1).isLeft(), isTrue);
    });
  });

  group('FuelType', () {
    test('maps api string', () {
      expect(FuelType.fromApi('electric'), FuelType.electric);
      expect(FuelType.fromApi('unknown'), FuelType.petrol);
    });

    test('apiValue round-trips', () {
      expect(FuelType.hybrid.apiValue, 'hybrid');
    });
  });
}
