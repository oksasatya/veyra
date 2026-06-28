import 'package:decimal/decimal.dart';
import 'package:veyra_mobile/features/expense/domain/entities/expense.dart';
import 'package:veyra_mobile/features/expense/domain/value_objects/expense_category.dart';

/// One category's share of the expense total, for the breakdown bar + legend.
class CategorySlice {
  const CategorySlice({
    required this.category,
    required this.amount,
    required this.fraction,
  });

  final ExpenseCategory category;
  final Decimal amount;

  /// 0..1 share of the grand total. 0 when the total is zero.
  final double fraction;
}

/// Display-only rollup of a vehicle's expenses: grand total, count, and the
/// per-category breakdown sorted largest-first. Pure — no I/O, no Flutter.
class ExpenseSummary {
  const ExpenseSummary({
    required this.total,
    required this.count,
    required this.slices,
    required this.highest,
  });

  final Decimal total;
  final int count;
  final List<CategorySlice> slices;
  final ExpenseCategory? highest;
}

/// Aggregate [expenses] into an [ExpenseSummary]. O(n) to sum + group by
/// category (a small fixed key set), then O(k log k) to sort the ≤5 slices —
/// k is the number of categories, so the sort is effectively constant. No N+1:
/// the list is already in memory from the list provider.
ExpenseSummary computeExpenseSummary(List<Expense> expenses) {
  final byCategory = <ExpenseCategory, Decimal>{};
  var total = Decimal.zero;
  for (final expense in expenses) {
    total += expense.amount;
    byCategory[expense.category] =
        (byCategory[expense.category] ?? Decimal.zero) + expense.amount;
  }

  final totalDouble = total.toDouble();
  final slices = byCategory.entries
      .map(
        (entry) => CategorySlice(
          category: entry.key,
          amount: entry.value,
          fraction: totalDouble == 0 ? 0 : entry.value.toDouble() / totalDouble,
        ),
      )
      .toList()
    ..sort((a, b) => b.amount.compareTo(a.amount));

  return ExpenseSummary(
    total: total,
    count: expenses.length,
    slices: slices,
    highest: slices.isEmpty ? null : slices.first.category,
  );
}
