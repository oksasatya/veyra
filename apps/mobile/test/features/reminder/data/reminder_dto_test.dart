import 'package:flutter_test/flutter_test.dart';
import 'package:veyra_mobile/features/reminder/data/models/reminder_dto.dart';
import 'package:veyra_mobile/features/reminder/domain/value_objects/reminder_type.dart';

void main() {
  group('ReminderDto.fromJson → toDomain', () {
    test('maps a full date reminder', () {
      final dto = ReminderDto.fromJson(const {
        'id': 'r1',
        'vehicle_id': 'v1',
        'title': 'Road tax renewal',
        'reminder_type': 'date',
        'due_date': '2026-07-01',
        'due_odometer': null,
        'is_completed': false,
        'notes': 'pay at samsat',
      });
      final reminder = dto.toDomain();

      expect(reminder.id, 'r1');
      expect(reminder.vehicleId, 'v1');
      expect(reminder.title, 'Road tax renewal');
      expect(reminder.type, ReminderType.date);
      expect(reminder.dueDate, DateTime.parse('2026-07-01'));
      expect(reminder.dueOdometer, isNull);
      expect(reminder.isCompleted, isFalse);
      expect(reminder.notes, 'pay at samsat');
    });

    test('maps an odometer reminder with a null due_date', () {
      final dto = ReminderDto.fromJson(const {
        'id': 'r2',
        'vehicle_id': 'v1',
        'title': 'Oil + filter service',
        'reminder_type': 'odometer',
        'due_date': null,
        'due_odometer': 129000,
        'is_completed': false,
        'notes': null,
      });
      final reminder = dto.toDomain();

      expect(reminder.type, ReminderType.odometer);
      expect(reminder.dueDate, isNull);
      expect(reminder.dueOdometer, 129000);
      expect(reminder.notes, isNull);
    });

    test('maps a completed both-type reminder', () {
      final dto = ReminderDto.fromJson(const {
        'id': 'r3',
        'vehicle_id': 'v1',
        'title': 'Insurance renewal',
        'reminder_type': 'both',
        'due_date': '2026-07-05',
        'due_odometer': 130000,
        'is_completed': true,
        'notes': null,
      });
      final reminder = dto.toDomain();

      expect(reminder.type, ReminderType.both);
      expect(reminder.dueDate, DateTime.parse('2026-07-05'));
      expect(reminder.dueOdometer, 130000);
      expect(reminder.isCompleted, isTrue);
    });

    test('falls back to date for an unknown reminder_type', () {
      final dto = ReminderDto.fromJson(const {
        'id': 'r4',
        'vehicle_id': 'v1',
        'title': 'Mystery',
        'reminder_type': 'wat',
        'due_date': '2026-07-05',
        'due_odometer': null,
        'is_completed': false,
        'notes': null,
      });

      expect(dto.toDomain().type, ReminderType.date);
    });
  });
}
