import 'package:flutter_test/flutter_test.dart';
import 'package:veyra_mobile/features/reminder/domain/usecases/validate_reminder.dart';
import 'package:veyra_mobile/features/reminder/domain/value_objects/reminder_type.dart';

void main() {
  final dueDate = DateTime(2026, 7, 1);

  group('validateReminder — title', () {
    test('rejects an empty title', () {
      final result = validateReminder(
        vehicleId: 'v1',
        title: '   ',
        type: ReminderType.date,
        dueDate: dueDate,
      );
      expect(result.isLeft(), isTrue);
      expect(
        result.getLeft().toNullable()?.field,
        'title',
      );
    });

    test('trims the title on success', () {
      final result = validateReminder(
        vehicleId: 'v1',
        title: '  Road tax  ',
        type: ReminderType.date,
        dueDate: dueDate,
      );
      expect(result.toNullable()?.title, 'Road tax');
    });
  });

  group('validateReminder — type=date', () {
    test('requires a dueDate', () {
      final result = validateReminder(
        vehicleId: 'v1',
        title: 'Road tax',
        type: ReminderType.date,
      );
      expect(result.isLeft(), isTrue);
      expect(result.getLeft().toNullable()?.field, 'dueDate');
    });

    test('accepts a dueDate and ignores a stray odometer', () {
      final result = validateReminder(
        vehicleId: 'v1',
        title: 'Road tax',
        type: ReminderType.date,
        dueDate: dueDate,
        dueOdometer: 99000,
      ).toNullable();
      expect(result?.dueDate, dueDate);
      expect(result?.dueOdometer, isNull);
    });
  });

  group('validateReminder — type=odometer', () {
    test('requires a dueOdometer', () {
      final result = validateReminder(
        vehicleId: 'v1',
        title: 'Oil change',
        type: ReminderType.odometer,
      );
      expect(result.isLeft(), isTrue);
      expect(result.getLeft().toNullable()?.field, 'dueOdometer');
    });

    test('rejects a negative dueOdometer', () {
      final result = validateReminder(
        vehicleId: 'v1',
        title: 'Oil change',
        type: ReminderType.odometer,
        dueOdometer: -1,
      );
      expect(result.isLeft(), isTrue);
      expect(result.getLeft().toNullable()?.field, 'dueOdometer');
    });

    test('accepts a dueOdometer and ignores a stray date', () {
      final result = validateReminder(
        vehicleId: 'v1',
        title: 'Oil change',
        type: ReminderType.odometer,
        dueDate: dueDate,
        dueOdometer: 129000,
      ).toNullable();
      expect(result?.dueOdometer, 129000);
      expect(result?.dueDate, isNull);
    });
  });

  group('validateReminder — type=both', () {
    test('requires both fields (missing date)', () {
      final result = validateReminder(
        vehicleId: 'v1',
        title: 'Insurance',
        type: ReminderType.both,
        dueOdometer: 130000,
      );
      expect(result.isLeft(), isTrue);
      expect(result.getLeft().toNullable()?.field, 'dueDate');
    });

    test('requires both fields (missing odometer)', () {
      final result = validateReminder(
        vehicleId: 'v1',
        title: 'Insurance',
        type: ReminderType.both,
        dueDate: dueDate,
      );
      expect(result.isLeft(), isTrue);
      expect(result.getLeft().toNullable()?.field, 'dueOdometer');
    });

    test('accepts when both fields are present', () {
      final result = validateReminder(
        vehicleId: 'v1',
        title: 'Insurance',
        type: ReminderType.both,
        dueDate: dueDate,
        dueOdometer: 130000,
      ).toNullable();
      expect(result?.dueDate, dueDate);
      expect(result?.dueOdometer, 130000);
    });
  });
}
