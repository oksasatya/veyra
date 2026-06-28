import 'package:decimal/decimal.dart';
import 'package:veyra_mobile/features/expense/domain/entities/expense.dart';
import 'package:veyra_mobile/features/expense/domain/value_objects/expense_category.dart';

/// Wire model for `ExpenseResponse`. Hand-mapped to the [Expense] entity.
/// `amount` arrives as a string → parsed to [Decimal] (never double for money).
class ExpenseDto {
  const ExpenseDto({
    required this.id,
    required this.vehicleId,
    required this.expenseDate,
    required this.category,
    required this.amount,
    required this.description,
  });

  factory ExpenseDto.fromJson(Map<String, dynamic> json) => ExpenseDto(
    id: json['id'] as String,
    vehicleId: json['vehicle_id'] as String,
    expenseDate: json['expense_date'] as String,
    category: json['category'] as String,
    amount: json['amount'] as String,
    description: json['description'] as String,
  );

  final String id;
  final String vehicleId;
  final String expenseDate;
  final String category;
  final String amount;
  final String description;

  Expense toDomain() => Expense(
    id: id,
    vehicleId: vehicleId,
    expenseDate: DateTime.parse(expenseDate),
    category: ExpenseCategory.fromApi(category),
    description: description,
    amount: Decimal.parse(amount),
  );
}
