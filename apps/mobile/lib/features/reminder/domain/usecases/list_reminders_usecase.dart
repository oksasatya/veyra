import 'package:fpdart/fpdart.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/features/reminder/domain/entities/reminder.dart';
import 'package:veyra_mobile/features/reminder/domain/repositories/reminder_repository.dart';

class ListRemindersUseCase {
  const ListRemindersUseCase(this._repo);
  final ReminderRepository _repo;

  Future<Either<Failure, List<Reminder>>> call(String vehicleId) =>
      _repo.list(vehicleId);
}
