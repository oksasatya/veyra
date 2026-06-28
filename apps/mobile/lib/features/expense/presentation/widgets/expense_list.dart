import 'package:decimal/decimal.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/core/error/failure_l10n.dart';
import 'package:veyra_mobile/core/theme/app_theme.dart';
import 'package:veyra_mobile/features/expense/domain/entities/expense.dart';
import 'package:veyra_mobile/features/expense/domain/value_objects/expense_category.dart';
import 'package:veyra_mobile/features/expense/presentation/controllers/expense_list_controller.dart';
import 'package:veyra_mobile/features/expense/presentation/expense_l10n.dart';
import 'package:veyra_mobile/l10n/app_localizations.dart';

/// Renders a vehicle's expenses (category icon/label · date · description,
/// trailing `Rp amount`) with loading/empty/error states.
class ExpenseList extends ConsumerWidget {
  const ExpenseList({required this.vehicleId, super.key});
  final String vehicleId;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final expenses = ref.watch(expenseListProvider(vehicleId));
    return expenses.when(
      loading: () => const _ExpenseSkeleton(),
      error: (e, _) => _ExpenseError(
        message: e is Failure
            ? localizedFailure(l10n, e)
            : l10n.expenseLoadError,
        onRetry: () => ref.invalidate(expenseListProvider(vehicleId)),
      ),
      data: (rows) => rows.isEmpty
          ? _ExpenseEmpty(l10n: l10n)
          : Column(
              children: [
                for (final e in rows) _ExpenseRow(expense: e),
              ],
            ),
    );
  }
}

class _ExpenseRow extends StatelessWidget {
  const _ExpenseRow({required this.expense});
  final Expense expense;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Container(
      padding: const EdgeInsets.symmetric(vertical: 13),
      decoration: const BoxDecoration(
        border: Border(bottom: BorderSide(color: VeyraColors.border)),
      ),
      child: Row(
        children: [
          _CategoryIcon(category: expense.category),
          const SizedBox(width: 13),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  localizedExpenseCategory(l10n, expense.category),
                  style: const TextStyle(
                    color: VeyraColors.text,
                    fontSize: 15,
                    fontWeight: FontWeight.w500,
                  ),
                ),
                const SizedBox(height: 3),
                Text(
                  '${_formatDate(expense.expenseDate)} · ${expense.description}',
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: plexMono(size: 12),
                ),
              ],
            ),
          ),
          const SizedBox(width: 10),
          Text(
            _money(expense.amount),
            style: plexMono(size: 14, color: VeyraColors.text),
          ),
        ],
      ),
    );
  }
}

class _CategoryIcon extends StatelessWidget {
  const _CategoryIcon({required this.category});
  final ExpenseCategory category;

  @override
  Widget build(BuildContext context) => Container(
    width: 40,
    height: 40,
    decoration: BoxDecoration(
      color: VeyraColors.surface2,
      borderRadius: BorderRadius.circular(11),
      border: Border.all(color: VeyraColors.border),
    ),
    child: Icon(_iconFor(category), size: 20, color: _colorFor(category)),
  );
}

class _ExpenseEmpty extends StatelessWidget {
  const _ExpenseEmpty({required this.l10n});
  final AppLocalizations l10n;

  @override
  Widget build(BuildContext context) => Container(
    padding: const EdgeInsets.symmetric(vertical: 32),
    alignment: Alignment.center,
    child: Text(
      l10n.expenseEmpty,
      style: const TextStyle(color: VeyraColors.textMuted, fontSize: 14),
    ),
  );
}

class _ExpenseSkeleton extends StatelessWidget {
  const _ExpenseSkeleton();

  @override
  Widget build(BuildContext context) => Column(
    children: [
      for (var i = 0; i < 3; i++)
        Container(
          height: 66,
          margin: const EdgeInsets.only(bottom: 1),
          decoration: const BoxDecoration(
            border: Border(bottom: BorderSide(color: VeyraColors.border)),
          ),
        ),
    ],
  );
}

class _ExpenseError extends StatelessWidget {
  const _ExpenseError({required this.message, required this.onRetry});
  final String message;
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: VeyraColors.surface,
        borderRadius: BorderRadius.circular(14),
        border: Border.all(color: VeyraColors.border),
      ),
      child: Row(
        children: [
          Expanded(
            child: Text(
              message,
              style: const TextStyle(
                color: VeyraColors.textMuted,
                fontSize: 13,
              ),
            ),
          ),
          TextButton(onPressed: onRetry, child: Text(l10n.commonRetry)),
        ],
      ),
    );
  }
}

IconData _iconFor(ExpenseCategory category) => switch (category) {
  ExpenseCategory.tire => Icons.trip_origin,
  ExpenseCategory.battery => Icons.battery_charging_full,
  ExpenseCategory.tax => Icons.receipt_long,
  ExpenseCategory.insurance => Icons.shield_outlined,
  ExpenseCategory.other => Icons.payments_outlined,
};

Color _colorFor(ExpenseCategory category) => switch (category) {
  ExpenseCategory.insurance => VeyraColors.accent,
  ExpenseCategory.tax => VeyraColors.info,
  ExpenseCategory.tire => VeyraColors.accentHover,
  _ => VeyraColors.textMuted,
};

const _months = [
  'Jan',
  'Feb',
  'Mar',
  'Apr',
  'May',
  'Jun',
  'Jul',
  'Aug',
  'Sep',
  'Oct',
  'Nov',
  'Dec',
];

String _formatDate(DateTime d) => '${d.day} ${_months[d.month - 1]}';

String _grouped(int n) {
  final s = n.toString();
  final buf = StringBuffer();
  for (var i = 0; i < s.length; i++) {
    if (i > 0 && (s.length - i) % 3 == 0) buf.write(',');
    buf.write(s[i]);
  }
  return buf.toString();
}

String _money(Decimal d) => 'Rp ${_grouped(d.round().toBigInt().toInt())}';
