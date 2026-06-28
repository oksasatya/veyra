import 'package:flutter_test/flutter_test.dart';
import 'package:veyra_mobile/features/document/domain/value_objects/expiry_status.dart';

void main() {
  final today = DateTime(2026, 6, 28);

  group('expiryStatusFor', () {
    test('null expiry → onFile', () {
      expect(
        expiryStatusFor(expiry: null, today: today),
        ExpiryStatus.onFile,
      );
    });

    test('past expiry → expired', () {
      expect(
        expiryStatusFor(expiry: DateTime(2026, 6, 27), today: today),
        ExpiryStatus.expired,
      );
    });

    test('expiry today → expiringSoon (date-only, not expired)', () {
      expect(
        expiryStatusFor(expiry: DateTime(2026, 6, 28), today: today),
        ExpiryStatus.expiringSoon,
      );
    });

    test('time-of-day on the same calendar day is ignored', () {
      expect(
        expiryStatusFor(
          expiry: DateTime(2026, 6, 28, 1),
          today: DateTime(2026, 6, 28, 23),
        ),
        ExpiryStatus.expiringSoon,
      );
    });

    test('within the 30-day window → expiringSoon', () {
      expect(
        expiryStatusFor(expiry: DateTime(2026, 7, 28), today: today),
        ExpiryStatus.expiringSoon,
      );
    });

    test('exactly 30 days away → expiringSoon (inclusive boundary)', () {
      expect(
        expiryStatusFor(expiry: DateTime(2026, 7, 28), today: today),
        ExpiryStatus.expiringSoon,
      );
    });

    test('31 days away → valid (just past the window)', () {
      expect(
        expiryStatusFor(expiry: DateTime(2026, 7, 29), today: today),
        ExpiryStatus.valid,
      );
    });

    test('far future → valid', () {
      expect(
        expiryStatusFor(expiry: DateTime(2027, 3, 12), today: today),
        ExpiryStatus.valid,
      );
    });

    test('labels are stable', () {
      expect(ExpiryStatus.expired.label, 'Expired');
      expect(ExpiryStatus.expiringSoon.label, 'Expiring soon');
      expect(ExpiryStatus.valid.label, 'Valid');
      expect(ExpiryStatus.onFile.label, 'On file');
    });
  });
}
