import 'package:fpdart/fpdart.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/features/reminder/domain/entities/reminder.dart';
import 'package:veyra_mobile/features/reminder/domain/value_objects/reminder_type.dart';

/// Validated input for creating a reminder. Built only after the cross-field
/// rule (see `validate_reminder.dart`) has accepted the values.
class CreateReminderInput {
  const CreateReminderInput({
    required this.vehicleId,
    required this.title,
    required this.type,
    this.dueDate,
    this.dueOdometer,
    this.notes,
  });

  final String vehicleId;
  final String title;
  final ReminderType type;
  final DateTime? dueDate;
  final int? dueOdometer;
  final String? notes;
}

/// Port: the reminder boundary the domain depends on.
abstract interface class ReminderRepository {
  Future<Either<Failure, List<Reminder>>> list(String vehicleId);
  Future<Either<Failure, Reminder>> create(CreateReminderInput input);
  Future<Either<Failure, Reminder>> complete(
    String vehicleId,
    String reminderId,
  );
}
