import 'package:decimal/decimal.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:veyra_mobile/features/fuel_log/data/models/fuel_log_dto.dart';

void main() {
  group('FuelLogDto', () {
    test('maps a full JSON payload to the domain entity', () {
      final dto = FuelLogDto.fromJson(const {
        'id': 'log-1',
        'vehicle_id': 'veh-1',
        'log_date': '2026-06-22',
        'odometer': 128450,
        'liters': '38.0',
        'price_per_liter': '14000',
        'total_cost': '532000',
        'station': 'Pertamina',
        'is_full_tank': true,
      });

      final log = dto.toDomain();

      expect(log.id, 'log-1');
      expect(log.vehicleId, 'veh-1');
      expect(log.logDate, DateTime(2026, 6, 22));
      expect(log.odometer, 128450);
      expect(log.liters, Decimal.parse('38.0'));
      expect(log.pricePerLiter, Decimal.fromInt(14000));
      expect(log.totalCost, Decimal.fromInt(532000));
      expect(log.station, 'Pertamina');
      expect(log.isFullTank, isTrue);
    });

    test('keeps decimal precision from string money fields', () {
      final dto = FuelLogDto.fromJson(const {
        'id': 'log-2',
        'vehicle_id': 'veh-1',
        'log_date': '2026-01-05',
        'odometer': 1000,
        'liters': '40.55',
        'price_per_liter': '13750.25',
        'total_cost': '557573.6375',
        'station': null,
        'is_full_tank': false,
      });

      final log = dto.toDomain();

      expect(log.liters, Decimal.parse('40.55'));
      expect(log.pricePerLiter, Decimal.parse('13750.25'));
      expect(log.totalCost, Decimal.parse('557573.6375'));
      expect(log.station, isNull);
      expect(log.isFullTank, isFalse);
    });
  });
}
