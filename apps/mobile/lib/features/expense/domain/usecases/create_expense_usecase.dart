import 'package:fpdart/fpdart.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/features/expense/domain/entities/expense.dart';
import 'package:veyra_mobile/features/expense/domain/repositories/expense_repository.dart';

class CreateExpenseUseCase {
  const CreateExpenseUseCase(this._repo);
  final ExpenseRepository _repo;

  Future<Either<Failure, Expense>> call(CreateExpenseInput input) =>
      _repo.create(input);
}
