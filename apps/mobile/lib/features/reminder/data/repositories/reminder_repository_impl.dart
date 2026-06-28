import 'package:dio/dio.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:fpdart/fpdart.dart';
import 'package:veyra_mobile/core/error/dio_error_mapper.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/core/network/dio_client.dart';
import 'package:veyra_mobile/features/reminder/data/datasources/reminder_remote_data_source.dart';
import 'package:veyra_mobile/features/reminder/domain/entities/reminder.dart';
import 'package:veyra_mobile/features/reminder/domain/repositories/reminder_repository.dart';
import 'package:veyra_mobile/features/reminder/domain/usecases/complete_reminder_usecase.dart';
import 'package:veyra_mobile/features/reminder/domain/usecases/create_reminder_usecase.dart';
import 'package:veyra_mobile/features/reminder/domain/usecases/list_reminders_usecase.dart';

class ReminderRepositoryImpl implements ReminderRepository {
  ReminderRepositoryImpl(this.remote);
  final ReminderRemoteDataSource remote;

  @override
  Future<Either<Failure, List<Reminder>>> list(String vehicleId) async {
    try {
      final dtos = await remote.list(vehicleId);
      return Right(dtos.map((d) => d.toDomain()).toList());
    } on DioException catch (e) {
      return Left(mapDioError(e));
    }
  }

  @override
  Future<Either<Failure, Reminder>> create(CreateReminderInput input) async {
    try {
      final dto = await remote.create(input.vehicleId, {
        'title': input.title,
        'reminder_type': input.type.apiValue,
        'due_date': _isoDate(input.dueDate),
        'due_odometer': input.dueOdometer,
        'notes': input.notes,
      });
      return Right(dto.toDomain());
    } on DioException catch (e) {
      return Left(mapDioError(e));
    }
  }

  @override
  Future<Either<Failure, Reminder>> complete(
    String vehicleId,
    String reminderId,
  ) async {
    try {
      final dto = await remote.complete(vehicleId, reminderId);
      return Right(dto.toDomain());
    } on DioException catch (e) {
      return Left(mapDioError(e));
    }
  }
}

/// Serialise a [DateTime] as a `YYYY-MM-DD` wire date (null passes through).
String? _isoDate(DateTime? date) {
  if (date == null) return null;
  final m = date.month.toString().padLeft(2, '0');
  final d = date.day.toString().padLeft(2, '0');
  return '${date.year}-$m-$d';
}

// ── Providers (DI) ───────────────────────────────────────────────────────────

final reminderRepositoryProvider = Provider<ReminderRepository>(
  (ref) =>
      ReminderRepositoryImpl(ReminderRemoteDataSource(ref.watch(dioProvider))),
);

final listRemindersUseCaseProvider = Provider<ListRemindersUseCase>(
  (ref) => ListRemindersUseCase(ref.watch(reminderRepositoryProvider)),
);

final createReminderUseCaseProvider = Provider<CreateReminderUseCase>(
  (ref) => CreateReminderUseCase(ref.watch(reminderRepositoryProvider)),
);

final completeReminderUseCaseProvider = Provider<CompleteReminderUseCase>(
  (ref) => CompleteReminderUseCase(ref.watch(reminderRepositoryProvider)),
);

/// Per-vehicle reminders, keyed by vehicle id. The add-sheet invalidates this
/// after a successful create, and the check control after a complete, so the
/// list refreshes.
final reminderListProvider = FutureProvider.family<List<Reminder>, String>((
  ref,
  vehicleId,
) async {
  final result = await ref.read(listRemindersUseCaseProvider)(vehicleId);
  return result.fold((failure) => throw failure, (reminders) => reminders);
});
