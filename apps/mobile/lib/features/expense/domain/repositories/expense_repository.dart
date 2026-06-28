import 'package:decimal/decimal.dart';
import 'package:fpdart/fpdart.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/features/expense/domain/entities/expense.dart';
import 'package:veyra_mobile/features/expense/domain/value_objects/expense_category.dart';

/// Validated input for creating an expense (the sheet builds this).
class CreateExpenseInput {
  const CreateExpenseInput({
    required this.vehicleId,
    required this.expenseDate,
    required this.category,
    required this.description,
    required this.amount,
  });

  final String vehicleId;
  final DateTime expenseDate;
  final ExpenseCategory category;
  final String description;
  final Decimal amount;
}

/// Port: the expense boundary the domain depends on.
abstract interface class ExpenseRepository {
  Future<Either<Failure, List<Expense>>> list(String vehicleId);
  Future<Either<Failure, Expense>> create(CreateExpenseInput input);
}
