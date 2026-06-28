import 'package:flutter_test/flutter_test.dart';
import 'package:veyra_mobile/features/expense/domain/value_objects/expense_category.dart';

void main() {
  group('ExpenseCategory', () {
    test('maps known api strings', () {
      expect(ExpenseCategory.fromApi('tire'), ExpenseCategory.tire);
      expect(ExpenseCategory.fromApi('battery'), ExpenseCategory.battery);
      expect(ExpenseCategory.fromApi('tax'), ExpenseCategory.tax);
      expect(ExpenseCategory.fromApi('insurance'), ExpenseCategory.insurance);
      expect(ExpenseCategory.fromApi('other'), ExpenseCategory.other);
    });

    test('falls back to other for unknown', () {
      expect(ExpenseCategory.fromApi('mystery'), ExpenseCategory.other);
      expect(ExpenseCategory.fromApi(''), ExpenseCategory.other);
    });

    test('apiValue round-trips the wire string', () {
      expect(ExpenseCategory.insurance.apiValue, 'insurance');
      expect(ExpenseCategory.tire.apiValue, 'tire');
    });

    test('exposes a human label', () {
      expect(ExpenseCategory.tax.label, 'Tax');
      expect(ExpenseCategory.other.label, 'Other');
    });
  });
}
