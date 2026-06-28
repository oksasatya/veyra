import 'package:decimal/decimal.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:veyra_mobile/features/expense/data/models/expense_dto.dart';
import 'package:veyra_mobile/features/expense/domain/value_objects/expense_category.dart';

void main() {
  group('ExpenseDto.fromJson → toDomain', () {
    final json = <String, dynamic>{
      'id': 'exp-1',
      'vehicle_id': 'veh-9',
      'expense_date': '2026-06-01',
      'category': 'insurance',
      'amount': '3400000.50',
      'description': 'annual premium',
    };

    test('parses wire keys into the DTO', () {
      final dto = ExpenseDto.fromJson(json);
      expect(dto.id, 'exp-1');
      expect(dto.vehicleId, 'veh-9');
      expect(dto.expenseDate, '2026-06-01');
      expect(dto.category, 'insurance');
      expect(dto.amount, '3400000.50');
      expect(dto.description, 'annual premium');
    });

    test('maps amount string to Decimal (precise, no double)', () {
      final expense = ExpenseDto.fromJson(json).toDomain();
      expect(expense.amount, Decimal.parse('3400000.50'));
    });

    test('maps expense_date ISO string to DateTime', () {
      final expense = ExpenseDto.fromJson(json).toDomain();
      expect(expense.expenseDate, DateTime.parse('2026-06-01'));
    });

    test('maps category string to ExpenseCategory enum', () {
      final expense = ExpenseDto.fromJson(json).toDomain();
      expect(expense.category, ExpenseCategory.insurance);
    });
  });
}
