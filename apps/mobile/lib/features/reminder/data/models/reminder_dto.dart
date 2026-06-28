import 'package:veyra_mobile/features/reminder/domain/entities/reminder.dart';
import 'package:veyra_mobile/features/reminder/domain/value_objects/reminder_type.dart';

/// Wire model for `ReminderResponse`. Hand-mapped to the [Reminder] entity.
/// `due_date` is a nullable ISO date string; `due_odometer` a nullable int.
class ReminderDto {
  const ReminderDto({
    required this.id,
    required this.vehicleId,
    required this.title,
    required this.reminderType,
    required this.isCompleted,
    this.dueDate,
    this.dueOdometer,
    this.notes,
  });

  factory ReminderDto.fromJson(Map<String, dynamic> json) => ReminderDto(
        id: json['id'] as String,
        vehicleId: json['vehicle_id'] as String,
        title: json['title'] as String,
        reminderType: json['reminder_type'] as String,
        isCompleted: json['is_completed'] as bool,
        dueDate: json['due_date'] as String?,
        dueOdometer: json['due_odometer'] as int?,
        notes: json['notes'] as String?,
      );

  final String id;
  final String vehicleId;
  final String title;
  final String reminderType;
  final String? dueDate;
  final int? dueOdometer;
  final bool isCompleted;
  final String? notes;

  Reminder toDomain() => Reminder(
        id: id,
        vehicleId: vehicleId,
        title: title,
        type: ReminderType.fromApi(reminderType),
        dueDate: dueDate == null ? null : DateTime.parse(dueDate!),
        dueOdometer: dueOdometer,
        isCompleted: isCompleted,
        notes: notes,
      );
}
