import 'package:fpdart/fpdart.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/features/reminder/domain/entities/reminder.dart';
import 'package:veyra_mobile/features/reminder/domain/repositories/reminder_repository.dart';

class CreateReminderUseCase {
  const CreateReminderUseCase(this._repo);
  final ReminderRepository _repo;

  Future<Either<Failure, Reminder>> call(CreateReminderInput input) =>
      _repo.create(input);
}
