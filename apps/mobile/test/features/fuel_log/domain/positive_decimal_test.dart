import 'package:decimal/decimal.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:veyra_mobile/features/fuel_log/domain/value_objects/positive_decimal.dart';

void main() {
  group('PositiveDecimal', () {
    test('accepts a positive decimal', () {
      final value = PositiveDecimal.create('38.0').toNullable();
      expect(value, Decimal.parse('38.0'));
    });

    test('trims surrounding whitespace', () {
      final value = PositiveDecimal.create('  14000 ').toNullable();
      expect(value, Decimal.fromInt(14000));
    });

    test('rejects zero', () {
      expect(PositiveDecimal.create('0').isLeft(), isTrue);
    });

    test('rejects a negative value', () {
      expect(PositiveDecimal.create('-5').isLeft(), isTrue);
    });

    test('rejects an unparseable string', () {
      expect(PositiveDecimal.create('abc').isLeft(), isTrue);
    });

    test('carries the field name in the failure', () {
      final failure =
          PositiveDecimal.create('0', field: 'liters').getLeft().toNullable();
      expect(failure?.field, 'liters');
    });
  });
}
