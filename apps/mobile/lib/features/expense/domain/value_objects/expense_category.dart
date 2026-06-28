/// Expense category — mirrors the backend's
/// `'tire' | 'battery' | 'tax' | 'insurance' | 'other'` string. `name` is the
/// wire value (`tire`, …).
enum ExpenseCategory {
  tire,
  battery,
  tax,
  insurance,
  other;

  static ExpenseCategory fromApi(String raw) => switch (raw) {
        'tire' => ExpenseCategory.tire,
        'battery' => ExpenseCategory.battery,
        'tax' => ExpenseCategory.tax,
        'insurance' => ExpenseCategory.insurance,
        _ => ExpenseCategory.other,
      };

  String get apiValue => name;

  String get label => switch (this) {
        ExpenseCategory.tire => 'Tire',
        ExpenseCategory.battery => 'Battery',
        ExpenseCategory.tax => 'Tax',
        ExpenseCategory.insurance => 'Insurance',
        ExpenseCategory.other => 'Other',
      };
}
