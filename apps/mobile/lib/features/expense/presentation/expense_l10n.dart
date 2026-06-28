import 'package:veyra_mobile/features/expense/domain/value_objects/expense_category.dart';
import 'package:veyra_mobile/l10n/app_localizations.dart';

/// Map an [ExpenseCategory] to its localized label.
String localizedExpenseCategory(AppLocalizations l10n, ExpenseCategory cat) =>
    switch (cat) {
      ExpenseCategory.tire => l10n.expenseCategoryTire,
      ExpenseCategory.battery => l10n.expenseCategoryBattery,
      ExpenseCategory.tax => l10n.expenseCategoryTax,
      ExpenseCategory.insurance => l10n.expenseCategoryInsurance,
      ExpenseCategory.other => l10n.expenseCategoryOther,
    };
