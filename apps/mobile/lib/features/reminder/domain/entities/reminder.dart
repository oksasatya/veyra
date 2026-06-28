import 'package:veyra_mobile/features/reminder/domain/value_objects/reminder_type.dart';

/// Domain entity — pure Dart. The data layer maps a DTO into this.
class Reminder {
  const Reminder({
    required this.id,
    required this.vehicleId,
    required this.title,
    required this.type,
    required this.isCompleted,
    this.dueDate,
    this.dueOdometer,
    this.notes,
  });

  final String id;
  final String vehicleId;
  final String title;
  final ReminderType type;
  final DateTime? dueDate;
  final int? dueOdometer;
  final bool isCompleted;
  final String? notes;
}
