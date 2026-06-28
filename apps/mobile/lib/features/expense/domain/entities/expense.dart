import 'package:decimal/decimal.dart';
import 'package:veyra_mobile/features/expense/domain/value_objects/expense_category.dart';

/// Domain entity — pure Dart. The data layer maps a DTO into this.
class Expense {
  const Expense({
    required this.id,
    required this.vehicleId,
    required this.expenseDate,
    required this.category,
    required this.description,
    required this.amount,
  });

  final String id;
  final String vehicleId;
  final DateTime expenseDate;
  final ExpenseCategory category;
  final String description;
  final Decimal amount;
}
