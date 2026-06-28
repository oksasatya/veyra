import 'package:dio/dio.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:fpdart/fpdart.dart';
import 'package:veyra_mobile/core/error/dio_error_mapper.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/core/network/dio_client.dart';
import 'package:veyra_mobile/features/expense/data/datasources/expense_remote_data_source.dart';
import 'package:veyra_mobile/features/expense/domain/entities/expense.dart';
import 'package:veyra_mobile/features/expense/domain/repositories/expense_repository.dart';
import 'package:veyra_mobile/features/expense/domain/usecases/create_expense_usecase.dart';
import 'package:veyra_mobile/features/expense/domain/usecases/list_expenses_usecase.dart';

class ExpenseRepositoryImpl implements ExpenseRepository {
  ExpenseRepositoryImpl(this.remote);
  final ExpenseRemoteDataSource remote;

  @override
  Future<Either<Failure, List<Expense>>> list(String vehicleId) async {
    try {
      final dtos = await remote.list(vehicleId);
      return Right(dtos.map((d) => d.toDomain()).toList());
    } on DioException catch (e) {
      return Left(mapDioError(e));
    }
  }

  @override
  Future<Either<Failure, Expense>> create(CreateExpenseInput input) async {
    try {
      final dto = await remote.create(input.vehicleId, {
        'expense_date': input.expenseDate.toIso8601String(),
        'category': input.category.apiValue,
        'description': input.description,
        'amount': input.amount.toString(),
      });
      return Right(dto.toDomain());
    } on DioException catch (e) {
      return Left(mapDioError(e));
    }
  }
}

// ── Providers (DI) ───────────────────────────────────────────────────────────

final expenseRepositoryProvider = Provider<ExpenseRepository>(
  (ref) =>
      ExpenseRepositoryImpl(ExpenseRemoteDataSource(ref.watch(dioProvider))),
);

final listExpensesUseCaseProvider = Provider<ListExpensesUseCase>(
  (ref) => ListExpensesUseCase(ref.watch(expenseRepositoryProvider)),
);

final createExpenseUseCaseProvider = Provider<CreateExpenseUseCase>(
  (ref) => CreateExpenseUseCase(ref.watch(expenseRepositoryProvider)),
);
