import 'package:decimal/decimal.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:veyra_mobile/features/expense/domain/entities/expense.dart';
import 'package:veyra_mobile/features/expense/domain/value_objects/expense_category.dart';
import 'package:veyra_mobile/features/fuel_log/domain/entities/fuel_log.dart';
import 'package:veyra_mobile/features/service_record/domain/entities/service_record.dart';
import 'package:veyra_mobile/features/vehicle/presentation/vehicle_activity.dart';

FuelLog _fuel(DateTime d) => FuelLog(
      id: 'f$d',
      vehicleId: 'v1',
      logDate: d,
      odometer: 1000,
      liters: Decimal.parse('10'),
      pricePerLiter: Decimal.parse('14000'),
      totalCost: Decimal.parse('140000'),
      isFullTank: true,
    );

ServiceRecord _service(DateTime d) => ServiceRecord(
      id: 's$d',
      vehicleId: 'v1',
      serviceDate: d,
      odometer: 1000,
      description: 'oil',
    );

Expense _expense(DateTime d) => Expense(
      id: 'e$d',
      vehicleId: 'v1',
      expenseDate: d,
      category: ExpenseCategory.tax,
      description: 'tax',
      amount: Decimal.parse('200000'),
    );

void main() {
  group('mergeActivity', () {
    test('merges all three streams newest-first', () {
      final items = mergeActivity(
        fuel: [_fuel(DateTime(2026, 6, 10))],
        services: [_service(DateTime(2026, 6, 20))],
        expenses: [_expense(DateTime(2026, 6, 1))],
      );
      expect(items.length, 3);
      expect(items[0], isA<ServiceActivity>()); // 20 Jun
      expect(items[1], isA<FuelActivity>()); // 10 Jun
      expect(items[2], isA<ExpenseActivity>()); // 1 Jun
    });

    test('caps the result at limit, keeping the newest', () {
      final items = mergeActivity(
        fuel: [
          _fuel(DateTime(2026, 6, 1)),
          _fuel(DateTime(2026, 6, 2)),
          _fuel(DateTime(2026, 6, 9)),
        ],
        services: const [],
        expenses: const [],
        limit: 2,
      );
      expect(items.length, 2);
      expect(items[0].date, DateTime(2026, 6, 9));
      expect(items[1].date, DateTime(2026, 6, 2));
    });

    test('empty streams → empty list', () {
      expect(
        mergeActivity(fuel: const [], services: const [], expenses: const []),
        isEmpty,
      );
    });
  });
}
