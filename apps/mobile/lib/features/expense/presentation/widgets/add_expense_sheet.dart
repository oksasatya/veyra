import 'package:decimal/decimal.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:veyra_mobile/core/error/failure_l10n.dart';
import 'package:veyra_mobile/core/theme/app_theme.dart';
import 'package:veyra_mobile/features/expense/data/repositories/expense_repository_impl.dart';
import 'package:veyra_mobile/features/expense/domain/repositories/expense_repository.dart';
import 'package:veyra_mobile/features/expense/domain/value_objects/expense_category.dart';
import 'package:veyra_mobile/features/expense/presentation/controllers/expense_list_controller.dart';
import 'package:veyra_mobile/l10n/app_localizations.dart';

/// Bottom-sheet form for logging an expense: date, category chips, description,
/// amount. Validates, calls create, pops on success.
class AddExpenseSheet extends ConsumerStatefulWidget {
  const AddExpenseSheet({required this.vehicleId, super.key});
  final String vehicleId;

  @override
  ConsumerState<AddExpenseSheet> createState() => _AddExpenseSheetState();
}

class _AddExpenseSheetState extends ConsumerState<AddExpenseSheet> {
  final _description = TextEditingController();
  final _amount = TextEditingController();
  ExpenseCategory _category = ExpenseCategory.other;
  DateTime _date = DateTime.now();
  String? _error;
  bool _saving = false;

  @override
  void dispose() {
    _description.dispose();
    _amount.dispose();
    super.dispose();
  }

  Future<void> _pickDate() async {
    final picked = await showDatePicker(
      context: context,
      initialDate: _date,
      firstDate: DateTime(2000),
      lastDate: DateTime(2100),
    );
    if (picked != null) {
      setState(() => _date = picked);
    }
  }

  Future<void> _submit() async {
    final l10n = AppLocalizations.of(context);
    if (_description.text.trim().isEmpty) {
      setState(() => _error = l10n.expenseErrorEnterDescription);
      return;
    }
    final amount = Decimal.tryParse(_amount.text.trim());
    if (amount == null || amount <= Decimal.zero) {
      setState(() => _error = l10n.expenseErrorInvalidAmount);
      return;
    }

    setState(() {
      _error = null;
      _saving = true;
    });
    final result = await ref.read(createExpenseUseCaseProvider)(
      CreateExpenseInput(
        vehicleId: widget.vehicleId,
        expenseDate: _date,
        category: _category,
        description: _description.text.trim(),
        amount: amount,
      ),
    );
    if (!mounted) return;
    final l10nAfter = AppLocalizations.of(context);
    result.fold(
      (failure) => setState(() {
        _error = localizedFailure(l10nAfter, failure);
        _saving = false;
      }),
      (_) {
        ref.invalidate(expenseListProvider(widget.vehicleId));
        Navigator.of(context).pop();
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final bottomInset = MediaQuery.of(context).viewInsets.bottom;
    return Padding(
      padding: EdgeInsets.only(bottom: bottomInset),
      child: Container(
        decoration: const BoxDecoration(
          color: VeyraColors.surface,
          borderRadius: BorderRadius.vertical(top: Radius.circular(26)),
          border: Border(top: BorderSide(color: VeyraColors.border)),
        ),
        padding: const EdgeInsets.fromLTRB(22, 0, 22, 26),
        child: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const _Grab(),
              Row(
                children: [
                  Text(l10n.expenseAddTitle, style: soraDisplay(size: 21)),
                  const Spacer(),
                  IconButton(
                    onPressed: () => Navigator.of(context).pop(),
                    icon: const Icon(Icons.close, color: VeyraColors.textMuted),
                  ),
                ],
              ),
              const SizedBox(height: 8),
              _Label(l10n.expenseFieldDate),
              _DateField(date: _date, onTap: _pickDate),
              const SizedBox(height: 14),
              _Label(l10n.expenseFieldCategory),
              _CategorySelector(
                value: _category,
                onChanged: (c) => setState(() => _category = c),
              ),
              const SizedBox(height: 14),
              _Label(l10n.expenseFieldDescription),
              TextField(
                controller: _description,
                decoration: InputDecoration(
                  hintText: l10n.expenseFieldDescriptionHint,
                ),
              ),
              const SizedBox(height: 14),
              _Label(l10n.expenseFieldAmount),
              TextField(
                controller: _amount,
                keyboardType: const TextInputType.numberWithOptions(
                  decimal: true,
                ),
                decoration: const InputDecoration(hintText: '0'),
              ),
              if (_error != null) ...[
                const SizedBox(height: 8),
                Text(
                  _error!,
                  style: const TextStyle(
                    color: VeyraColors.danger,
                    fontSize: 13,
                  ),
                ),
              ],
              const SizedBox(height: 22),
              FilledButton(
                onPressed: _saving ? null : _submit,
                child: _saving
                    ? const SizedBox(
                        height: 22,
                        width: 22,
                        child: CircularProgressIndicator(
                          strokeWidth: 2.4,
                          color: VeyraColors.bg,
                        ),
                      )
                    : Text(l10n.expenseSave),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _Grab extends StatelessWidget {
  const _Grab();

  @override
  Widget build(BuildContext context) => Container(
    width: 40,
    height: 5,
    margin: const EdgeInsets.fromLTRB(0, 12, 0, 6),
    decoration: BoxDecoration(
      color: const Color(0xFF39424F),
      borderRadius: BorderRadius.circular(3),
    ),
  );
}

class _DateField extends StatelessWidget {
  const _DateField({required this.date, required this.onTap});
  final DateTime date;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) => InkWell(
    onTap: onTap,
    borderRadius: BorderRadius.circular(14),
    child: Container(
      height: 54,
      padding: const EdgeInsets.symmetric(horizontal: 16),
      decoration: BoxDecoration(
        color: VeyraColors.surface,
        borderRadius: BorderRadius.circular(14),
        border: Border.all(color: VeyraColors.border),
      ),
      alignment: Alignment.centerLeft,
      child: Text(
        '${date.day} ${_months[date.month - 1]} ${date.year}',
        style: plexMono(size: 16, color: VeyraColors.text),
      ),
    ),
  );
}

class _Label extends StatelessWidget {
  const _Label(this.text);
  final String text;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.only(bottom: 8),
    child: Text(
      text,
      style: const TextStyle(
        color: VeyraColors.textMuted,
        fontSize: 13,
        fontWeight: FontWeight.w500,
      ),
    ),
  );
}

class _CategorySelector extends StatelessWidget {
  const _CategorySelector({required this.value, required this.onChanged});
  final ExpenseCategory value;
  final ValueChanged<ExpenseCategory> onChanged;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Wrap(
      spacing: 8,
      runSpacing: 8,
      children: [
        for (final c in ExpenseCategory.values)
          ChoiceChip(
            label: Text(_localizedCategory(l10n, c)),
            selected: c == value,
            onSelected: (_) => onChanged(c),
            backgroundColor: VeyraColors.surface,
            selectedColor: VeyraColors.accent,
            labelStyle: TextStyle(
              color: c == value ? VeyraColors.bg : VeyraColors.text,
              fontWeight: FontWeight.w500,
            ),
            side: const BorderSide(color: VeyraColors.border),
          ),
      ],
    );
  }
}

String _localizedCategory(AppLocalizations l10n, ExpenseCategory cat) =>
    switch (cat) {
      ExpenseCategory.tire => l10n.expenseCategoryTire,
      ExpenseCategory.battery => l10n.expenseCategoryBattery,
      ExpenseCategory.tax => l10n.expenseCategoryTax,
      ExpenseCategory.insurance => l10n.expenseCategoryInsurance,
      ExpenseCategory.other => l10n.expenseCategoryOther,
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
