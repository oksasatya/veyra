import 'package:fpdart/fpdart.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/features/reminder/domain/entities/reminder.dart';
import 'package:veyra_mobile/features/reminder/domain/repositories/reminder_repository.dart';

class CompleteReminderUseCase {
  const CompleteReminderUseCase(this._repo);
  final ReminderRepository _repo;

  Future<Either<Failure, Reminder>> call(String vehicleId, String reminderId) =>
      _repo.complete(vehicleId, reminderId);
}
