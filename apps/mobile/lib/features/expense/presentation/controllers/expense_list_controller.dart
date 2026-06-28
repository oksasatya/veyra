import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:veyra_mobile/features/expense/data/repositories/expense_repository_impl.dart';
import 'package:veyra_mobile/features/expense/domain/entities/expense.dart';

/// Per-vehicle expense list, keyed by vehicle id. A thrown Failure surfaces as
/// the `AsyncValue` error state for the UI. The sheet re-fetches via
/// `ref.invalidate(expenseListProvider(vehicleId))` after a successful create.
final expenseListProvider =
    FutureProvider.family<List<Expense>, String>((ref, vehicleId) async {
  final result = await ref.read(listExpensesUseCaseProvider)(vehicleId);
  return result.fold((failure) => throw failure, (expenses) => expenses);
});
