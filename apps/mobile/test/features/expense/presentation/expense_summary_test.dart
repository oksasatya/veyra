import 'package:decimal/decimal.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:veyra_mobile/features/expense/domain/entities/expense.dart';
import 'package:veyra_mobile/features/expense/domain/value_objects/expense_category.dart';
import 'package:veyra_mobile/features/expense/presentation/expense_summary.dart';

Expense _expense(ExpenseCategory category, String amount) => Expense(
      id: 'e-$category-$amount',
      vehicleId: 'v1',
      expenseDate: DateTime(2026, 6, 1),
      category: category,
      description: 'x',
      amount: Decimal.parse(amount),
    );

void main() {
  group('computeExpenseSummary', () {
    test('empty list → zero total, no slices, null highest', () {
      final summary = computeExpenseSummary([]);
      expect(summary.total, Decimal.zero);
      expect(summary.count, 0);
      expect(summary.slices, isEmpty);
      expect(summary.highest, isNull);
    });

    test('sums total, counts rows, groups by category', () {
      final summary = computeExpenseSummary([
        _expense(ExpenseCategory.insurance, '4000000'),
        _expense(ExpenseCategory.tax, '2000000'),
        _expense(ExpenseCategory.insurance, '0'),
        _expense(ExpenseCategory.tire, '2000000'),
      ]);
      expect(summary.total, Decimal.parse('8000000'));
      expect(summary.count, 4);
      expect(summary.slices.length, 3); // insurance, tax, tire
    });

    test('slices sorted largest-first; highest is the top category', () {
      final summary = computeExpenseSummary([
        _expense(ExpenseCategory.tax, '2000000'),
        _expense(ExpenseCategory.insurance, '4000000'),
        _expense(ExpenseCategory.tire, '1000000'),
      ]);
      expect(
        summary.slices.map((s) => s.category).toList(),
        [ExpenseCategory.insurance, ExpenseCategory.tax, ExpenseCategory.tire],
      );
      expect(summary.highest, ExpenseCategory.insurance);
    });

    test('fraction is each category share of the total', () {
      final summary = computeExpenseSummary([
        _expense(ExpenseCategory.insurance, '7500000'),
        _expense(ExpenseCategory.tax, '2500000'),
      ]);
      final insurance =
          summary.slices.firstWhere((s) => s.category == ExpenseCategory.insurance);
      final tax = summary.slices.firstWhere((s) => s.category == ExpenseCategory.tax);
      expect(insurance.fraction, closeTo(0.75, 1e-9));
      expect(tax.fraction, closeTo(0.25, 1e-9));
    });
  });
}
