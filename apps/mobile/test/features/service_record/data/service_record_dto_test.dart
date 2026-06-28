import 'package:decimal/decimal.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:veyra_mobile/features/service_record/data/models/service_record_dto.dart';

void main() {
  group('ServiceRecordDto.toDomain', () {
    test('maps all fields with a present cost', () {
      final dto = ServiceRecordDto.fromJson(<String, dynamic>{
        'id': 'rec-1',
        'vehicle_id': 'veh-9',
        'service_date': '2026-06-15',
        'odometer': 52000,
        'description': 'Oil change',
        'workshop': 'AutoCare',
        'cost': '350000.50',
        'notes': 'Synthetic oil',
      });

      final record = dto.toDomain();

      expect(record.id, 'rec-1');
      expect(record.vehicleId, 'veh-9');
      expect(record.serviceDate, DateTime.parse('2026-06-15'));
      expect(record.odometer, 52000);
      expect(record.description, 'Oil change');
      expect(record.workshop, 'AutoCare');
      expect(record.cost, Decimal.parse('350000.50'));
      expect(record.notes, 'Synthetic oil');
    });

    test('maps a null cost to a null Decimal', () {
      final dto = ServiceRecordDto.fromJson(<String, dynamic>{
        'id': 'rec-2',
        'vehicle_id': 'veh-9',
        'service_date': '2026-01-02',
        'odometer': 10000,
        'description': 'Tire rotation',
        'workshop': null,
        'cost': null,
        'notes': null,
      });

      final record = dto.toDomain();

      expect(record.cost, isNull);
      expect(record.workshop, isNull);
      expect(record.notes, isNull);
      expect(record.serviceDate, DateTime.parse('2026-01-02'));
    });

    test('preserves monetary precision via Decimal (not double)', () {
      final dto = ServiceRecordDto.fromJson(<String, dynamic>{
        'id': 'rec-3',
        'vehicle_id': 'veh-9',
        'service_date': '2026-03-03',
        'odometer': 20000,
        'description': 'Brake pads',
        'cost': '199999.99',
      });

      expect(dto.toDomain().cost, Decimal.parse('199999.99'));
    });
  });
}
