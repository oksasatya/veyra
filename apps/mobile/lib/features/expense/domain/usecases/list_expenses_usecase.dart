import 'package:fpdart/fpdart.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/features/expense/domain/entities/expense.dart';
import 'package:veyra_mobile/features/expense/domain/repositories/expense_repository.dart';

class ListExpensesUseCase {
  const ListExpensesUseCase(this._repo);
  final ExpenseRepository _repo;

  Future<Either<Failure, List<Expense>>> call(String vehicleId) =>
      _repo.list(vehicleId);
}
