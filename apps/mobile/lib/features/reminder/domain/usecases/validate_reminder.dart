import 'package:fpdart/fpdart.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/features/reminder/domain/repositories/reminder_repository.dart';
import 'package:veyra_mobile/features/reminder/domain/value_objects/reminder_type.dart';

/// Domain rule: a reminder's due fields depend on its [ReminderType].
///
/// - `date`     ⇒ [dueDate] required (odometer ignored)
/// - `odometer` ⇒ [dueOdometer] required, non-negative (date ignored)
/// - `both`     ⇒ both required
///
/// Returns the normalised [CreateReminderInput] (irrelevant fields nulled out)
/// or a [ValidationFailure] naming the offending field.
Either<ValidationFailure, CreateReminderInput> validateReminder({
  required String vehicleId,
  required String title,
  required ReminderType type,
  DateTime? dueDate,
  int? dueOdometer,
  String? notes,
}) {
  final trimmedTitle = title.trim();
  if (trimmedTitle.isEmpty) {
    return const Left(
      ValidationFailure('Enter a title for the reminder.', field: 'title'),
    );
  }

  final needsDate = type == ReminderType.date || type == ReminderType.both;
  final needsOdometer =
      type == ReminderType.odometer || type == ReminderType.both;

  if (needsDate && dueDate == null) {
    return const Left(
      ValidationFailure('A due date is required.', field: 'dueDate'),
    );
  }
  if (needsOdometer) {
    if (dueOdometer == null) {
      return const Left(
        ValidationFailure(
          'A due odometer reading is required.',
          field: 'dueOdometer',
        ),
      );
    }
    if (dueOdometer < 0) {
      return const Left(
        ValidationFailure(
          'Odometer cannot be negative.',
          field: 'dueOdometer',
        ),
      );
    }
  }

  final trimmedNotes = notes?.trim();
  return Right(
    CreateReminderInput(
      vehicleId: vehicleId,
      title: trimmedTitle,
      type: type,
      dueDate: needsDate ? dueDate : null,
      dueOdometer: needsOdometer ? dueOdometer : null,
      notes: (trimmedNotes == null || trimmedNotes.isEmpty)
          ? null
          : trimmedNotes,
    ),
  );
}
